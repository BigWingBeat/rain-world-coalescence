use std::io::Read;

use bevy::{
    ecs::{component::Component, system::Query},
    log::error,
    prelude::{Deref, DerefMut},
};
use bytes::Bytes;
use serde::de::DeserializeOwned;

use crate::{
    serde::{deserialize_packet_from, ByteQueue, Header, MalformedHeader},
    state::ConnectionStateImpl,
    Error,
};

/// A component holding a buffer for packets of the specified type that have been received from the peer
#[derive(Debug, Component, Deref, DerefMut)]
pub struct Received<T> {
    pub buffer: Vec<T>,
}

impl<T> Default for Received<T> {
    fn default() -> Self {
        Self {
            buffer: Default::default(),
        }
    }
}

/// A component that receives and deserializes packets from a peer
#[derive(Debug, Component, Default)]
pub struct PacketReceiver {
    /// Bytes that were received from the peer, but have yet to be deserialized into strongly-typed packets
    receive_queue: ByteQueue,
    /// Set to `Some()` when we have deserialized a packet header, but haven't received the rest of the packet yet
    pending_header: Option<Result<Header, MalformedHeader>>,
}

impl PacketReceiver {
    pub fn new() -> Self {
        Self::default()
    }

    /// Receive some bytes from the peer
    pub fn receive(&mut self, bytes: Bytes) {
        self.receive_queue.push(bytes);
    }

    /// Poll for packets received from the peer, ignoring (but not discarding) packets that do not match the specified state T
    ///
    /// Returns Ok(Some(T::Packet)) if a packet was successfully received.
    ///
    /// Returns Ok(None) if no packets are currently available.
    ///
    /// If an error was encountered while receiving a packet, returns Err(_) and discards the packet
    pub(crate) fn poll_receive<T>(&mut self) -> Result<Option<T::Packet>, Error>
    where
        T: ConnectionStateImpl,
        T::Packet: DeserializeOwned,
    {
        let header = match self.pending_header.take() {
            Some(header) => header,
            None => {
                if self.receive_queue.len() < Header::ENCODED_LEN {
                    // Wait until we have enough bytes to decode the header
                    return Ok(None);
                }

                let mut buf = [0; Header::ENCODED_LEN];
                self.receive_queue.read_exact(&mut buf).unwrap();
                Header::decode(buf)
            }
        };

        let payload_len = header
            .as_ref()
            .map_or_else(|header| header.length, |header| header.length);

        if self.receive_queue.len() < payload_len {
            // Haven't received all of the serialized packet data yet, so do nothing and wait
            self.pending_header = Some(header);
            return Ok(None);
        }

        // We have the entire packet payload, deserialize or discard it as appropriate
        match header {
            Ok(header) => {
                if T::STATE == header.state {
                    // The packet has a matching state, deserialize it

                    // Use `.take()` to limit the bytes that can be read so in case the deserialization goes wrong it can't
                    // eat into the data of subsequent packets
                    let mut reader = (&mut self.receive_queue).take(payload_len as _);
                    match deserialize_packet_from(&mut reader) {
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
                            self.receive_queue.discard_bytes(remaining);
                            Err(e)
                        }
                    }
                } else {
                    // The packet has a valid but mismatched state, do nothing
                    self.pending_header = Some(Ok(header));
                    Ok(None)
                }
            }
            // The packet has an invalid state, discard it
            Err(header) => {
                self.receive_queue.discard_bytes(payload_len);
                Err(header.error)
            }
        }
    }
}

pub(crate) fn packet_deserialize<S>(
    mut query: Query<(&mut PacketReceiver, &mut Received<S::Packet>, &mut S)>,
) where
    S: ConnectionStateImpl + Send + Sync + 'static,
    S::Packet: DeserializeOwned + Send + Sync + 'static,
{
    for (mut receiver, mut received_buf, mut state) in query.iter_mut() {
        if let Err(e) = match receiver.poll_receive::<S>() {
            Ok(Some(packet)) => state.handle_packet_deserialize(&packet).map(|_| {
                received_buf.push(packet);
            }),
            Ok(None) => Ok(()),
            Err(e) => Err(e),
        } {
            error!("Error while receiving packet for state {}: {e}", S::STATE)
        }
    }
}
