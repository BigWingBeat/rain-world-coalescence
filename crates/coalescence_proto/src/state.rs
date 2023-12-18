use std::{any::Any, fmt::Debug};

use enumset::EnumSet;
use strum::{Display, EnumCount};

use crate::{
    packet::{HandshakePacket, HandshakePacketDiscriminant, LobbyPacket},
    Error, ErrorKind,
};

#[derive(Debug, Display, PartialEq, Eq, EnumCount)]
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

/// An object-safe trait for storing [`ConnectionStateImpl`]s as trait objects
pub(crate) trait ConnectionStateObject: Any + Debug {
    fn state(&self) -> ConnectionState;
}

/// A trait for the internal logic of a specified connection state, including transitions to other states
pub(crate) trait ConnectionStateImpl<T>: ConnectionStateObject {
    const STATE: ConnectionState;

    fn handle_packet_serialize(&mut self, packet: &T);
    fn handle_packet_deserialize(&mut self, packet: &T);

    fn poll_state_change(&self) -> Option<Box<dyn ConnectionStateObject>>;
}

// impl<T, P> ConnectionStateImpl<P> for &mut T
// where
//     T: ConnectionStateImpl<P>,
// {
//     const STATE: ConnectionState = T::STATE;

//     fn handle_packet_serialize(&mut self, packet: &P) {
//         self.handle_packet_serialize(packet)
//     }

//     fn handle_packet_deserialize(&mut self, packet: &P) {
//         self.handle_packet_deserialize(packet)
//     }

//     fn poll_state_change(&self) -> Option<Box<dyn ConnectionStateObject>> {
//         self.poll_state_change()
//     }
// }

// impl<T> ConnectionStateObject for &mut T
// where
//     T: ConnectionStateObject,
// {
//     fn state(&self) -> ConnectionState {
//         self.state()
//     }
// }

/// In order to complete the handshake, every handshake packet type must be handled at least once
#[derive(Debug, Default)]
pub struct HandshakeState {
    handled: EnumSet<HandshakePacketDiscriminant>,
}

impl ConnectionStateImpl<HandshakePacket<'_>> for HandshakeState {
    const STATE: ConnectionState = ConnectionState::Handshake;

    fn handle_packet_serialize(&mut self, packet: &HandshakePacket<'_>) {
        self.handled.insert(packet.into());
    }

    fn handle_packet_deserialize(&mut self, packet: &HandshakePacket<'_>) {
        self.handled.insert(packet.into());
    }

    fn poll_state_change(&self) -> Option<Box<dyn ConnectionStateObject>> {
        (self.handled == EnumSet::ALL).then(|| Box::new(LobbyState) as _)
    }
}

impl ConnectionStateObject for HandshakeState {
    fn state(&self) -> ConnectionState {
        Self::STATE
    }
}

#[derive(Debug, Default)]
pub struct LobbyState;

impl ConnectionStateImpl<LobbyPacket<'_>> for LobbyState {
    const STATE: ConnectionState = ConnectionState::Lobby;

    fn handle_packet_serialize(&mut self, packet: &LobbyPacket<'_>) {}

    fn handle_packet_deserialize(&mut self, packet: &LobbyPacket<'_>) {}

    fn poll_state_change(&self) -> Option<Box<dyn ConnectionStateObject>> {
        None
    }
}

impl ConnectionStateObject for LobbyState {
    fn state(&self) -> ConnectionState {
        Self::STATE
    }
}
