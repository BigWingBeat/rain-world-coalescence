//! The different types of packets that correspond to each of the different [`state`]s that the connection can be in

use enumset::EnumSetType;
use serde::{Deserialize, Serialize};
use strum::EnumDiscriminants;

use crate::{
    connection::Channel,
    peer::{Client, Server},
    state::{ConnectionStateImpl, HandshakeState, LobbyState},
};

/// A type that can represent any of the packet types associated with a certain connection state
pub(crate) trait PacketSet: Sized {
    type State: ConnectionStateImpl<Self>;
}

/// A packet type that is included in the packet set `T`
///
/// The generic type parameter `P` is the [`Peer`] on which this packet type can be serialized and transmitted (i.e. is "outbound")
pub trait Packet<T, P> {
    const CHANNEL: Channel;
    fn into_set(self) -> T;
}

/// The packets that needs to be exchanged between the client and server while establishing the connection
#[derive(Debug, Serialize, Deserialize, EnumDiscriminants)]
#[strum_discriminants(derive(EnumSetType), enumset(no_super_impls))]
#[strum_discriminants(name(HandshakePacketDiscriminant))]
pub enum HandshakePacket<'a> {
    #[serde(borrow)]
    Profile(Profile<'a>),
    Lobby(Lobby<'a>),
}

impl PacketSet for HandshakePacket<'_> {
    type State = HandshakeState;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Profile<'a> {
    pub username: &'a str,
}

impl<'a> Packet<HandshakePacket<'a>, Client> for Profile<'a> {
    const CHANNEL: Channel = Channel::Ordered;

    fn into_set(self) -> HandshakePacket<'a> {
        HandshakePacket::Profile(self)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Lobby<'a> {
    #[serde(borrow)]
    pub usernames: Vec<&'a str>,
}

impl<'a> Packet<HandshakePacket<'a>, Server> for Lobby<'a> {
    const CHANNEL: Channel = Channel::Ordered;

    fn into_set(self) -> HandshakePacket<'a> {
        HandshakePacket::Lobby(self)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum LobbyPacket<'a> {
    #[serde(borrow)]
    PlayerJoined(PlayerJoined<'a>),
    PlayerLeft(PlayerLeft),
    Disconnect,
}

impl PacketSet for LobbyPacket<'_> {
    type State = LobbyState;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerJoined<'a> {
    pub username: &'a str,
}

impl<'a> Packet<LobbyPacket<'a>, Server> for PlayerJoined<'a> {
    const CHANNEL: Channel = Channel::Ordered;

    fn into_set(self) -> LobbyPacket<'a> {
        LobbyPacket::PlayerJoined(self)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerLeft;

impl<'a> Packet<LobbyPacket<'a>, Server> for PlayerLeft {
    const CHANNEL: Channel = Channel::Ordered;

    fn into_set(self) -> LobbyPacket<'a> {
        LobbyPacket::PlayerLeft(self)
    }
}
