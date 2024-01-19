use std::marker::PhantomData;

use bevy::{ecs::component::Component, utils::default};
use bytes::Bytes;

use crate::{
    packet::{Channel, Packet},
    serde::{serialize_into, Header},
    state::ConnectionStateImpl,
    Error,
};

/// A component for sending packets to a peer
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

    fn buffer_for_channel(&mut self, channel: Channel) -> &mut Vec<Bytes> {
        match channel {
            Channel::Ordered => &mut self.ordered_buffer,
            Channel::Unordered => &mut self.unordered_buffer,
            Channel::Unreliable => &mut self.unreliable_buffer,
        }
    }

    /// Serialize a packet and send it to the peer
    pub fn send<T>(&mut self, packet: T) -> Result<(), Error>
    where
        T: Packet<P>,
    {
        let packet = packet.into_set();
        let header = Header::new(<T::State as ConnectionStateImpl>::STATE, &packet)?;

        let mut bytes = Vec::with_capacity(Header::ENCODED_LEN + header.length);
        bytes.extend_from_slice(&header.encode());
        serialize_into(&mut bytes, &packet)?;

        let buffer = self.buffer_for_channel(T::CHANNEL);
        buffer.push(bytes.into());

        Ok(())
    }

    /// Take all of the bytes to be sent to the peer over a specified channel
    pub fn take_bytes(&mut self, channel: Channel) -> Vec<Bytes> {
        std::mem::take(self.buffer_for_channel(channel))
    }
}
