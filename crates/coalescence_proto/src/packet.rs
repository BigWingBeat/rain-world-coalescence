//! The different types of packets that correspond to each of the different [`state`]s that the connection can be in

use enumset::EnumSetType;
use serde::{Deserialize, Serialize};
use strum::EnumDiscriminants;

use crate::{
    peer::{Client, Server},
    state::{HandshakeHandler, LobbyHandler, PacketHandler},
};

/// A packet with an associated handler type
pub trait Packet: Sized {
    type Handler: PacketHandler<Self>;
}

/// A type that can be converted into the packet type `T`
///
/// The generic type parameter `P` is the [`Peer`] on which this packet type can be serialized and transmitted (i.e. is "outbound")
pub trait IntoPacket<T, P> {
    fn into_packet(self) -> T;
}

/// Reflexive implementation of `IntoPacket` for `T: Packet` so that both payloads and packets can be passed to [`Connection::serialize`]
impl<T, P> IntoPacket<T, P> for T
where
    T: Packet,
{
    fn into_packet(self) -> Self {
        self
    }
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

impl Packet for HandshakePacket<'_> {
    type Handler = HandshakeHandler;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Profile<'a> {
    pub username: &'a str,
}

impl<'a> IntoPacket<HandshakePacket<'a>, Client> for Profile<'a> {
    fn into_packet(self) -> HandshakePacket<'a> {
        HandshakePacket::Profile(self)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Lobby<'a> {
    #[serde(borrow)]
    pub usernames: Vec<&'a str>,
}

impl<'a> IntoPacket<HandshakePacket<'a>, Server> for Lobby<'a> {
    fn into_packet(self) -> HandshakePacket<'a> {
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

impl Packet for LobbyPacket<'_> {
    type Handler = LobbyHandler;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerJoined<'a> {
    pub username: &'a str,
}

impl<'a> IntoPacket<LobbyPacket<'a>, Server> for PlayerJoined<'a> {
    fn into_packet(self) -> LobbyPacket<'a> {
        LobbyPacket::PlayerJoined(self)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerLeft;

impl<'a> IntoPacket<LobbyPacket<'a>, Server> for PlayerLeft {
    fn into_packet(self) -> LobbyPacket<'a> {
        LobbyPacket::PlayerLeft(self)
    }
}
