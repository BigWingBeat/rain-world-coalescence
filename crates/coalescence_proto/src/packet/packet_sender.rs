use std::marker::PhantomData;

use bevy::{ecs::component::Component, utils::default};
use bytes::Bytes;

use crate::{
    channel::{Channel, Ordered, Unordered, Unreliable},
    peer::Outbound,
    serde::{serialize_into, serialized_size},
    Error, Is,
};

use super::{AnyPacket, OrderedHeader, Packet};

/// A component for serializing packets and sending them over the network
///
/// The generic type parameter `P` is the type of *this* peer, not the remote peer that packets are sent to
#[derive(Debug, Component)]
pub struct PacketSender<P> {
    ordered_buffer: Vec<Bytes>,
    unordered_buffer: Vec<Bytes>,
    unreliable_buffer: Vec<Bytes>,
    peer: PhantomData<P>,
}

impl<P> Default for PacketSender<P> {
    fn default() -> Self {
        Self::new()
    }
}

impl<P> PacketSender<P> {
    pub fn new() -> Self {
        Self {
            ordered_buffer: default(),
            unordered_buffer: default(),
            unreliable_buffer: default(),
            peer: PhantomData,
        }
    }

    fn buffer_for_channel<C: Channel>(&mut self) -> &mut Vec<Bytes> {
        if C::is::<Ordered>() {
            &mut self.ordered_buffer
        } else if C::is::<Unordered>() {
            &mut self.unordered_buffer
        } else if C::is::<Unreliable>() {
            &mut self.unreliable_buffer
        } else {
            unreachable!("There should only be 3 channel types: Ordered, Unordered and Unreliable, but an unexpected fourth channel type exists: '{}'", std::any::type_name::<C>())
        }
    }

    /// Serialize the given packet and send it over the network
    pub fn send<T>(&mut self, packet: T) -> Result<(), Error>
    where
        T: Packet,
        T::Direction: Outbound<P>,
    {
        let packet: AnyPacket = packet.into();
        let length = serialized_size(&packet)?;

        let mut bytes = if T::Channel::is::<Ordered>() {
            let mut bytes = Vec::with_capacity(length + OrderedHeader::ENCODED_LEN);
            let header = OrderedHeader { length };
            header.encode_into(&mut bytes)?;
            bytes
        } else {
            Vec::with_capacity(length)
        };

        serialize_into(&mut bytes, &packet)?;

        self.buffer_for_channel::<T::Channel>().push(bytes.into());

        Ok(())
    }

    /// Take all of the bytes currently buffered to be sent over the specified channel
    pub fn take_bytes<C: Channel>(&mut self) -> Vec<Bytes> {
        std::mem::take(self.buffer_for_channel::<C>())
    }
}
