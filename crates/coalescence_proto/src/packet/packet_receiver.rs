use std::io::Read;

use bevy::ecs::{
    component::Component,
    entity::Entity,
    event::{Event, EventWriter},
    system::Query,
};
use bytes::Bytes;

use crate::{
    channel::{Channel, Ordered, Unordered, Unreliable},
    serde::{deserialize, deserialize_from, ByteQueue},
    Error, Is,
};

use super::{AnyPacket, OrderedHeader, ReceivedPackets};

/// An event that is sent whenever an error is encountered while receiving a packet
#[derive(Debug, Event)]
pub struct ReceiveError {
    pub entity: Entity,
    pub error: Error,
}

/// A component that receives bytes from the network and deserializes them into packets
#[derive(Debug, Component, Default)]
pub struct PacketReceiver {
    /// Bytes received from the ordered-reliable channel, that have yet to be deserialized into packets
    ordered_queue: ByteQueue,
    /// Set to `Some()` when we have deserialized a packet header from the ordered-reliable channel,
    /// but haven't received the rest of the packet yet
    pending_ordered_header: Option<OrderedHeader>,
    /// Packets received from the unordered-reliable channel, that haven't been deserialized yet
    unordered_buffer: Vec<Bytes>,
    /// Packets received from the unordered-unreliable channel, that haven't been deserialized yet
    unreliable_buffer: Vec<Bytes>,
}

impl PacketReceiver {
    pub fn new() -> Self {
        Self::default()
    }

    /// Receive some bytes from the specified channel.
    ///
    /// For the ordered-reliable channel, repeated calls to this method must guarantee that the order the bytes are passed in
    /// over the course of the calls corresponds exactly to the order the bytes were received in, with no gaps or reordering.
    /// This guarantee does not apply to the other channels.
    pub fn receive<C: Channel>(&mut self, bytes: Bytes) {
        if C::is::<Ordered>() {
            self.ordered_queue.push(bytes);
        } else if C::is::<Unordered>() {
            self.unordered_buffer.push(bytes);
        } else if C::is::<Unreliable>() {
            self.unreliable_buffer.push(bytes);
        } else {
            unreachable!("There should only be 3 channel types: Ordered, Unordered and Unreliable, but an unexpected fourth channel type exists: '{}'", std::any::type_name::<C>())
        }
    }

    /// Poll for packets received from the ordered-reliable channel.
    ///
    /// Returns Ok(Some(AnyPacket)) if a packet was successfully deserialized.
    ///
    /// Returns Ok(None) if no packets are currently available.
    ///
    /// If an error was encountered while receiving a packet, returns Err(_) and discards the packet
    pub(crate) fn poll_ordered_reliable(&mut self) -> Result<Option<AnyPacket>, Error> {
        let header = match self.pending_ordered_header.take() {
            Some(header) => header,
            None => {
                if self.ordered_queue.len() < OrderedHeader::ENCODED_LEN {
                    // Wait until we have enough bytes to decode the header
                    return Ok(None);
                }

                let mut buf = [0; OrderedHeader::ENCODED_LEN];
                self.ordered_queue.read_exact(&mut buf).unwrap();
                OrderedHeader::decode(buf)
            }
        };

        if self.ordered_queue.len() < header.length {
            // Haven't received all of the serialized packet data yet, so do nothing and wait
            self.pending_ordered_header = Some(header);
            return Ok(None);
        }

        // We have the entire packet payload, deserialize or discard it as appropriate

        // Use `.take()` to limit the bytes that can be read so in case the deserialization goes wrong it can't
        // eat into the data of subsequent packets
        let mut reader = (&mut self.ordered_queue).take(header.length as _);
        match deserialize_from(&mut reader) {
            Ok(packet) => {
                assert_eq!(
                    reader.limit(),
                    0,
                    "Deserializing packet didn't read all of its bytes"
                );

                Ok(Some(packet))
            }
            Err(e) => {
                // A deserialization error means this packet's data is most likely malformed and unusable, so we discard
                // any of it that has been left over from the deserialization potentially having halted prematurely
                let remaining = reader.limit() as _;
                self.ordered_queue.discard_bytes(remaining);
                Err(e)
            }
        }
    }
}

pub(crate) fn receive(
    mut query: Query<(Entity, &mut PacketReceiver, ReceivedPackets)>,
    mut errors: EventWriter<ReceiveError>,
) {
    for (entity, mut receiver, mut buffers) in query.iter_mut() {
        // Deserialize the buffered unordered and unreliable packets together, as they're handled identically here
        let PacketReceiver {
            unordered_buffer,
            unreliable_buffer,
            ..
        } = &mut *receiver;

        for packets in unordered_buffer
            .drain(..)
            .chain(unreliable_buffer.drain(..))
        {
            match deserialize(&packets) {
                Ok(packet) => buffers.receive(packet),
                Err(error) => {
                    errors.send(ReceiveError { entity, error });
                }
            }
        }

        // Deserialize any ordered packets that are available
        while let Some(result) = receiver.poll_ordered_reliable().transpose() {
            match result {
                Ok(packet) => buffers.receive(packet),
                Err(error) => {
                    errors.send(ReceiveError { entity, error });
                }
            }
        }
    }
}
