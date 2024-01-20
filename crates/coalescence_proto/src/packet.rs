//! The different types of packets that correspond to each of the different [`state`]s that the connection can be in

use bevy::ecs::bundle::Bundle;
use enumset::EnumSetType;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use strum::EnumDiscriminants;

use crate::{
    peer::{Client, Server},
    state::{ConnectionStateImpl, HandshakeState, LobbyState},
};

mod receiver;
mod sender;

pub(crate) use receiver::packet_deserialize;
pub use receiver::{PacketReceiver, Received};
pub use sender::PacketSender;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Channel {
    /// Packets are guranteed to arrive in the same order they are sent
    Ordered,
    /// Packets are guranteed to arrive, but possibly in a different order than they were sent
    Unordered,
    /// Packets may be lost, and may arrive in a different order than they were sent
    Unreliable,
}

/// A type that can represent any of a number of different packet types, and is suitable for being serialized and deserialized
pub trait PacketSet: Serialize + DeserializeOwned + 'static {}

/// A packet type that can be converted to a specified packet set
///
/// The generic type parameter `P` is the [`Peer`] on which this packet type can be serialized and transmitted (i.e. is "outbound")
pub trait Packet<P> {
    /// The packet set that this packet is included in
    type Set: PacketSet;

    type State: ConnectionStateImpl<Packet = Self::Set>;

    /// The channel which this packet should be sent over
    const CHANNEL: Channel;

    /// Convert this packet into its associated packet set
    fn into_set(self) -> Self::Set;
}

#[derive(Debug, Bundle, Default)]
pub struct ReceivedPackets {
    handshake: Received<HandshakePacket>,
    lobby: Received<LobbyPacket>,
}

/// The packets that needs to be exchanged between the client and server while establishing the connection
#[derive(Debug, Serialize, Deserialize, EnumDiscriminants)]
#[strum_discriminants(derive(EnumSetType), enumset(no_super_impls))]
#[strum_discriminants(name(HandshakePacketDiscriminant))]
pub enum HandshakePacket {
    // #[serde(borrow)]
    Profile(Profile),
    Lobby(Lobby),
}

impl PacketSet for HandshakePacket {}

#[derive(Debug, Serialize, Deserialize)]
pub struct Profile {
    pub username: String,
}

impl Packet<Client> for Profile {
    type Set = HandshakePacket;
    type State = HandshakeState;
    const CHANNEL: Channel = Channel::Ordered;

    fn into_set(self) -> HandshakePacket {
        HandshakePacket::Profile(self)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Lobby {
    // #[serde(borrow)]
    pub usernames: Vec<String>,
}

impl Packet<Server> for Lobby {
    type Set = HandshakePacket;
    type State = HandshakeState;
    const CHANNEL: Channel = Channel::Ordered;

    fn into_set(self) -> HandshakePacket {
        HandshakePacket::Lobby(self)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum LobbyPacket {
    // #[serde(borrow)]
    PlayerJoined(PlayerJoined),
    PlayerLeft(PlayerLeft),
    Disconnect,
}

impl PacketSet for LobbyPacket {}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerJoined {
    pub username: String,
}

impl Packet<Server> for PlayerJoined {
    type Set = LobbyPacket;
    type State = LobbyState;
    const CHANNEL: Channel = Channel::Ordered;

    fn into_set(self) -> LobbyPacket {
        LobbyPacket::PlayerJoined(self)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerLeft;

impl Packet<Server> for PlayerLeft {
    type Set = LobbyPacket;
    type State = LobbyState;
    const CHANNEL: Channel = Channel::Ordered;

    fn into_set(self) -> LobbyPacket {
        LobbyPacket::PlayerLeft(self)
    }
}
