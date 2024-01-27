//! The different types of packets that correspond to each of the different [`state`]s that the connection can be in
#![allow(non_snake_case)]

use bevy::{
    ecs::{component::Component, query::QueryData},
    prelude::{Deref, DerefMut},
};
use serde::{Deserialize, Serialize};

use crate::{
    channel::{Channel, Ordered},
    peer::{Bidirectional, ClientToServer, Direction, ServerToClient},
};

mod header;
mod packet_receiver;
mod packet_sender;

pub(crate) use header::OrderedHeader;
pub(crate) use packet_receiver::receive;
pub use packet_receiver::{PacketReceiver, ReceiveError};
pub use packet_sender::PacketSender;

/// A macro for generating repetitive code that is needed for each packet type
macro_rules! all_packets {
    ($( $packet:ident ),+) => {
        /// An enum that can represent any packet type
        #[derive(Debug, Serialize, Deserialize)]
        pub enum AnyPacket {
            $( $packet($packet), )+
        }

        $(
            impl From<$packet> for AnyPacket {
                fn from(packet: $packet) -> Self {
                    Self::$packet(packet)
                }
            }
        )+

        /// A helper [`SystemParam`] for sorting [`AnyPacket`]s into the correct [`Received`] buffer
        #[derive(Debug, QueryData)]
        #[query_data(mutable)]
        pub(crate) struct ReceivedPackets {
            $(
                $packet: &'static mut Received<$packet>,
            )+
        }

        impl ReceivedPacketsItem<'_> {
            fn receive(&mut self, packet: AnyPacket) {
                match packet {
                    $(
                        AnyPacket::$packet(packet) => self.$packet.buffer.push(packet),
                    )+
                }
            }
        }
    };
}

all_packets!(Profile, Lobby, PlayerJoined, PlayerLeft, Disconnect);

/// A component holding a buffer for packets of the specified type that have been received from the peer
#[derive(Debug, Component, Deref, DerefMut)]
pub struct Received<T> {
    pub buffer: Vec<T>,
}

impl<T> Default for Received<T> {
    fn default() -> Self {
        Self {
            buffer: Default::default(),
        }
    }
}

/// A type that can be serialized and transmitted over the network
pub trait Packet: Into<AnyPacket> {
    /// The channel that this packet should be transmitted over
    type Channel: Channel;

    /// The direction between the client and server that this packet is valid to be sent over
    type Direction: Direction;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Profile {
    pub username: String,
}

impl Packet for Profile {
    type Channel = Ordered;
    type Direction = ClientToServer;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Lobby {
    pub usernames: Vec<String>,
}

impl Packet for Lobby {
    type Channel = Ordered;
    type Direction = ServerToClient;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerJoined {
    pub username: String,
}

impl Packet for PlayerJoined {
    type Channel = Ordered;
    type Direction = ServerToClient;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerLeft;

impl Packet for PlayerLeft {
    type Channel = Ordered;
    type Direction = ServerToClient;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Disconnect;

impl Packet for Disconnect {
    type Channel = Ordered;
    type Direction = Bidirectional;
}
