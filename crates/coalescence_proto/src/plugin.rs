use std::marker::PhantomData;

use bevy::{
    app::{App, Plugin, Update},
    ecs::{
        bundle::Bundle,
        schedule::{IntoSystemConfigs, SystemSet},
    },
};

use crate::{
    packet::{receive, PacketReceiver, PacketSender},
    peer::Peer,
};

#[derive(Debug, Bundle, Default)]
pub struct ConnectionBundle<P: Peer> {
    sender: PacketSender<P>,
    receiver: PacketReceiver,
    // received_packets: ReceivedPackets,
}

#[derive(Debug, SystemSet, Hash, PartialEq, Eq, Clone, Copy)]
pub struct SendPackets;

#[derive(Debug, SystemSet, Hash, PartialEq, Eq, Clone, Copy)]
pub struct ReceivePackets;

#[derive(Debug)]
pub struct ProtoPlugin<P>(PhantomData<P>);

impl<P: Peer> Plugin for ProtoPlugin<P> {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, receive.in_set(ReceivePackets));
    }
}
