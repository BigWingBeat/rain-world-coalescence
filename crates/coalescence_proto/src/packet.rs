//! The different types of packets that correspond to each of the different [`state`]s that the connection can be in

use enumset::EnumSetType;
use serde::{Deserialize, Serialize};
use strum::EnumDiscriminants;

use crate::{
    connection::Channel,
    peer::{Client, Server},
};

/// A type that can represent any of the packet types associated with a certain connection state
pub trait PacketSet {}

/// A packet type that is included in the packet set `T`
///
/// The generic type parameter `P` is the [`Peer`] on which this packet type can be serialized and transmitted (i.e. is "outbound")
pub trait Packet<T, P> {
    /// The channel which this packet should be sent over
    const CHANNEL: Channel;

    /// Convert this packet into its associated packet set
    fn into_set(self) -> T;
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

impl Packet<HandshakePacket, Client> for Profile {
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

impl Packet<HandshakePacket, Server> for Lobby {
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

impl Packet<LobbyPacket, Server> for PlayerJoined {
    const CHANNEL: Channel = Channel::Ordered;

    fn into_set(self) -> LobbyPacket {
        LobbyPacket::PlayerJoined(self)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerLeft;

impl Packet<LobbyPacket, Server> for PlayerLeft {
    const CHANNEL: Channel = Channel::Ordered;

    fn into_set(self) -> LobbyPacket {
        LobbyPacket::PlayerLeft(self)
    }
}
