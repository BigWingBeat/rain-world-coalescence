use std::marker::PhantomData;

use bevy::{
    app::{App, Plugin, Update},
    ecs::{
        component::Component,
        entity::Entity,
        event::EventWriter,
        schedule::{IntoSystemConfigs, SystemSet},
        system::Query,
    },
    log::error,
};
use serde::de::DeserializeOwned;

use crate::state::{ConnectionStateImpl, HandshakeState, LobbyState};

// #[derive(Debug, Bundle)]
// pub struct ProtoConnectionBundle<P: Send + Sync + 'static> {
//     connection: Connection<P>,
// }

#[derive(Debug, SystemSet, Hash, PartialEq, Eq, Clone, Copy)]
pub struct DeserializePackets;

#[derive(Debug)]
pub struct ProtoPlugin<P>(PhantomData<P>);

impl<P: Send + Sync + 'static> Plugin for ProtoPlugin<P> {
    fn build(&self, app: &mut App) {
        // app.add_systems(
        //     Update,
        //     (
        //         packet_deserialize::<HandshakeState, P>,
        //         packet_deserialize::<LobbyState, P>,
        //     )
        //         .in_set(DeserializePackets),
        // );
    }
}
