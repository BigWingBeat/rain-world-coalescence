use std::marker::PhantomData;

use bevy::{
    app::{App, Plugin, Update},
    ecs::{
        bundle::Bundle,
        schedule::{IntoSystemConfigs, SystemSet},
    },
};

use crate::{
    packet::{packet_deserialize, PacketReceiver, PacketSender, ReceivedPackets},
    state::{DefaultState, HandshakeState, LobbyState},
    Peer,
};

#[derive(Debug, Bundle, Default)]
pub struct ConnectionBundle<P: Peer> {
    sender: PacketSender<P>,
    receiver: PacketReceiver,
    received_packets: ReceivedPackets,
    state: DefaultState,
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
