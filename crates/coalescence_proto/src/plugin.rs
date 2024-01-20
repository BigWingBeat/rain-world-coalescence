use std::marker::PhantomData;

use bevy::{
    app::{App, Plugin, Update},
    ecs::{
        schedule::{IntoSystemConfigs, SystemSet},
        system::EntityCommands,
    },
};

use crate::{
    packet::{
        packet_deserialize, HandshakePacket, LobbyPacket, PacketReceiver, PacketSender, Received,
    },
    state::{default_state, HandshakeState, LobbyState},
    Peer,
};

pub trait EntityCommandsExt {
    fn insert_connection<P: Peer>(&mut self) -> &mut Self;
    fn try_insert_connection<P: Peer>(&mut self) -> &mut Self;
}

impl EntityCommandsExt for EntityCommands<'_, '_, '_> {
    fn insert_connection<P: Peer>(&mut self) -> &mut Self {
        self.insert((
            PacketSender::<P>::default(),
            PacketReceiver::default(),
            Received::<HandshakePacket>::default(),
            Received::<LobbyPacket>::default(),
            default_state(),
        ))
    }

    fn try_insert_connection<P: Peer>(&mut self) -> &mut Self {
        self.try_insert((
            PacketSender::<P>::default(),
            PacketReceiver::default(),
            Received::<HandshakePacket>::default(),
            Received::<LobbyPacket>::default(),
            default_state(),
        ))
    }
}

#[derive(Debug, SystemSet, Hash, PartialEq, Eq, Clone, Copy)]
pub struct SendPackets;

#[derive(Debug, SystemSet, Hash, PartialEq, Eq, Clone, Copy)]
pub struct ReceivePackets;

#[derive(Debug)]
pub struct ProtoPlugin<P>(PhantomData<P>);

impl<P: Peer> Plugin for ProtoPlugin<P> {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                packet_deserialize::<HandshakeState>,
                packet_deserialize::<LobbyState>,
            )
                .in_set(ReceivePackets),
        );
    }
}
