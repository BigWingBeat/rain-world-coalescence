use enumset::EnumSet;
use strum::{Display, EnumDiscriminants};
use thiserror::Error;

use crate::packet::{HandshakePacket, HandshakePacketDiscriminant, LobbyPacket};

#[derive(Debug, Error)]
#[error("Expected the connection to be in state {expected}, but it was in state {actual} instead")]
pub struct WrongStateError {
    expected: ConnectionState,
    actual: ConnectionState,
}

#[derive(Debug, Default, EnumDiscriminants)]
#[strum_discriminants(derive(Display))]
#[strum_discriminants(name(ConnectionState))]
pub enum ConnectionStateHandler {
    #[default]
    Negotiation,
    Authentication,
    Handshake(HandshakeHandler),
    Lobby(LobbyHandler),
    Established,
}

pub trait PacketHandler<T> {
    fn try_from_state(state: &mut ConnectionStateHandler) -> Result<&mut Self, WrongStateError>;

    fn handle_packet_serialize(&mut self, packet: &T);
    fn handle_packet_deserialize(&mut self, packet: &T);

    fn poll_state_change(&self) -> Option<ConnectionStateHandler>;
}

/// In order to complete the handshake, every handshake packet type must be handled at least once
#[derive(Debug, Default)]
pub struct HandshakeHandler {
    handled: EnumSet<HandshakePacketDiscriminant>,
}

impl PacketHandler<HandshakePacket<'_>> for HandshakeHandler {
    fn try_from_state(state: &mut ConnectionStateHandler) -> Result<&mut Self, WrongStateError> {
        match state {
            ConnectionStateHandler::Handshake(handshake) => Ok(handshake),
            other => Err(WrongStateError {
                expected: ConnectionState::Handshake,
                actual: (&*other).into(),
            }),
        }
    }

    fn handle_packet_serialize(&mut self, packet: &HandshakePacket<'_>) {
        self.handled.insert(packet.into());
    }

    fn handle_packet_deserialize(&mut self, packet: &HandshakePacket<'_>) {
        self.handled.insert(packet.into());
    }

    fn poll_state_change(&self) -> Option<ConnectionStateHandler> {
        (self.handled == EnumSet::ALL).then_some(ConnectionStateHandler::Lobby(LobbyHandler))
    }
}

#[derive(Debug, Default)]
pub struct LobbyHandler;

impl PacketHandler<LobbyPacket<'_>> for LobbyHandler {
    fn try_from_state(state: &mut ConnectionStateHandler) -> Result<&mut Self, WrongStateError> {
        match state {
            ConnectionStateHandler::Lobby(lobby) => Ok(lobby),
            other => Err(WrongStateError {
                expected: ConnectionState::Lobby,
                actual: (&*other).into(),
            }),
        }
    }

    fn handle_packet_serialize(&mut self, packet: &LobbyPacket<'_>) {}

    fn handle_packet_deserialize(&mut self, packet: &LobbyPacket<'_>) {}

    fn poll_state_change(&self) -> Option<ConnectionStateHandler> {
        None
    }
}
