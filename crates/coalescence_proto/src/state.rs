use std::{any::Any, fmt::Debug};

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

/// A trait object for downcasting, similar to `Any`, but that allows getting the underlying concrete type's type name, instead
/// of just the type ID
pub(crate) trait NamedAny {
    /// Manually converts to a `&dyn Any` trait object, which is necessary because trait object upcasting is unstable
    fn upcast(&self) -> &dyn Any;

    /// Manually converts to a `&mut dyn Any` trait object, which is necessary because trait object upcasting is unstable
    fn upcast_mut(&mut self) -> &mut dyn Any;

    /// The `std::any::type_name()` of the underlying concrete type
    fn type_name(&self) -> &'static str;
}

impl<T: Any> NamedAny for T {
    fn upcast(&self) -> &dyn Any {
        self
    }

    fn upcast_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn type_name(&self) -> &'static str {
        std::any::type_name::<T>()
    }
}

impl dyn NamedAny + '_ {
    /// Returns some reference to the inner value if it is of type `T`, or `None` if it isn't.
    fn downcast_ref<T: Any>(&self) -> Option<&T> {
        self.upcast().downcast_ref()
    }
}

/// A trait for the internal logic of a specified connection state, including transitions to other states
pub(crate) trait ConnectionStateImpl: Debug {
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

/// An object-safe trait for storing [`ConnectionStateImpl`]s as trait objects
pub(crate) trait ConnectionStateObject: Debug {
    /// The corresponding state enum value that this impl represents
    fn state(&self) -> ConnectionState;

    /// Handle a packet being serialized and transmitted
    fn handle_packet_serialize(&mut self, packet: &dyn NamedAny) -> Result<(), Error>;

    /// Handle a packet being received and deserialized
    fn handle_packet_deserialize(&mut self, packet: &dyn NamedAny) -> Result<(), Error>;

    /// Check if the connection should transition into a new state
    fn poll_state_change(&self) -> Option<Box<dyn ConnectionStateObject>>;
}

/// Impl to allow storing and using `ConnectionStateImpl`s as `ConnectionStateObject` trait objects
impl<T> ConnectionStateObject for T
where
    T: ConnectionStateImpl + Debug,
    T::Packet: 'static,
{
    fn state(&self) -> ConnectionState {
        T::STATE
    }

    fn handle_packet_serialize(&mut self, packet: &dyn NamedAny) -> Result<(), Error> {
        packet
            .downcast_ref::<T::Packet>()
            .ok_or_else(|| {
                ErrorKind::WrongPacket {
                    state: T::STATE,
                    expected: std::any::type_name::<T::Packet>(),
                    actual: packet.type_name(),
                }
                .into()
            })
            .and_then(|packet| self.handle_packet_serialize(packet))
    }

    fn handle_packet_deserialize(&mut self, packet: &dyn NamedAny) -> Result<(), Error> {
        packet
            .downcast_ref::<T::Packet>()
            .ok_or_else(|| {
                ErrorKind::WrongPacket {
                    state: T::STATE,
                    expected: std::any::type_name::<T::Packet>(),
                    actual: packet.type_name(),
                }
                .into()
            })
            .and_then(|packet| self.handle_packet_deserialize(packet))
    }

    fn poll_state_change(&self) -> Option<Box<dyn ConnectionStateObject>> {
        self.poll_state_change()
            .map(|state| -> Box<dyn ConnectionStateObject> { Box::new(state) })
    }
}

/// In order to complete the handshake, every handshake packet type must be handled at least once
#[derive(Debug, Default)]
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

    fn poll_state_change(&self) -> Option<impl ConnectionStateImpl + 'static> {
        (self.handled == EnumSet::ALL).then_some(LobbyState)
    }
}

#[derive(Debug, Default)]
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

    fn poll_state_change(&self) -> Option<impl ConnectionStateImpl + 'static> {
        None::<LobbyState>
    }
}

pub(crate) fn default_state() -> Box<dyn ConnectionStateObject> {
    Box::<HandshakeState>::default()
}
