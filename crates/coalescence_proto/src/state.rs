use std::fmt::Debug;

use bevy::ecs::component::Component;
use enumset::EnumSet;
use strum::{Display, EnumCount};

use crate::{
    packet::{HandshakePacket, HandshakePacketDiscriminant, LobbyPacket},
    Error, ErrorKind,
};

/// A simple C-style enum that describes all possible states a connection can be in
#[derive(Debug, Display, PartialEq, Eq, Clone, Copy, EnumCount)]
pub enum ConnectionState {
    Negotiation,
    Authentication,
    Handshake,
    Lobby,
    Established,
    Disconnecting,
}

impl From<ConnectionState> for u8 {
    fn from(state: ConnectionState) -> Self {
        state as u8
    }
}

impl TryFrom<u8> for ConnectionState {
    type Error = Error;

    fn try_from(state: u8) -> Result<Self, Self::Error> {
        // SAFETY: We only transmute if the byte is in-bounds for the enum, so it will never produce an invalid enum value
        ((state as usize) < Self::COUNT)
            .then(|| unsafe { std::mem::transmute(state) })
            .ok_or(ErrorKind::InvalidState(state).into())
    }
}

/// A trait for the internal logic of a specified connection state, including transitions to other states
pub(crate) trait ConnectionStateImpl: Component {
    /// The packet type that is handled by this state
    type Packet;

    /// The corresponding state enum value that this impl represents
    const STATE: ConnectionState;

    /// Handle a packet being serialized and transmitted
    fn handle_packet_serialize(&mut self, packet: &Self::Packet) -> Result<(), Error>;

    /// Handle a packet being received and deserialized
    fn handle_packet_deserialize(&mut self, packet: &Self::Packet) -> Result<(), Error>;

    /// Check if the connection should transition into a new state
    fn poll_state_change(&self) -> Option<impl ConnectionStateImpl + 'static>;
}

pub(crate) type DefaultState = HandshakeState;

/// In order to complete the handshake, every handshake packet type must be handled at least once
#[derive(Debug, Default, Component)]
pub struct HandshakeState {
    handled: EnumSet<HandshakePacketDiscriminant>,
}

impl ConnectionStateImpl for HandshakeState {
    type Packet = HandshakePacket;
    const STATE: ConnectionState = ConnectionState::Handshake;

    fn handle_packet_serialize(&mut self, packet: &HandshakePacket) -> Result<(), Error> {
        self.handled.insert(packet.into());
        Ok(())
    }

    fn handle_packet_deserialize(&mut self, packet: &HandshakePacket) -> Result<(), Error> {
        self.handled.insert(packet.into());
        Ok(())
    }

    fn poll_state_change(&self) -> Option<impl ConnectionStateImpl> {
        (self.handled == EnumSet::ALL).then_some(LobbyState)
    }
}

#[derive(Debug, Default, Component)]
pub struct LobbyState;

impl ConnectionStateImpl for LobbyState {
    type Packet = LobbyPacket;
    const STATE: ConnectionState = ConnectionState::Lobby;

    fn handle_packet_serialize(&mut self, _packet: &LobbyPacket) -> Result<(), Error> {
        Ok(())
    }

    fn handle_packet_deserialize(&mut self, _packet: &LobbyPacket) -> Result<(), Error> {
        Ok(())
    }

    fn poll_state_change(&self) -> Option<impl ConnectionStateImpl> {
        None::<LobbyState>
    }
}
