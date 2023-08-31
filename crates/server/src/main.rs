use std::net::{IpAddr, Ipv6Addr};

use bevy::prelude::*;
use bevy_quinnet::{
    server::{
        certificate::CertificateRetrievalMode, ConnectionEvent, QuinnetServerPlugin, Server,
        ServerConfiguration,
    },
    shared::channel::ChannelId,
};
use multiplayer_mvp_common::{Handshake, DEFAULT_PORT};

fn main() {
    App::new()
        .add_plugins((MinimalPlugins, QuinnetServerPlugin::default()))
        .add_systems(Startup, start_listening)
        .add_systems(Update, receive_messages)
        .run();
}

fn start_listening(mut server: ResMut<Server>) {
    server
        .start_endpoint(
            ServerConfiguration::from_ip(IpAddr::V6(Ipv6Addr::UNSPECIFIED), DEFAULT_PORT),
            CertificateRetrievalMode::GenerateSelfSigned {
                server_hostname: "Rain World Multiplayer MVP".to_string(),
            },
        )
        .unwrap();
}

fn receive_messages(mut server: ResMut<Server>, mut new_connections: EventReader<ConnectionEvent>) {
    let endpoint = server.endpoint_mut();
    for client_id in endpoint.clients() {
        while endpoint
            .try_receive_message_from::<Handshake>(client_id)
            .is_some()
        {
            println!("Received handshake from client {client_id}");
        }
    }

    for &ConnectionEvent { id } in new_connections.iter() {
        println!("New connection from client {id}, sending handshake...");
        endpoint
            .send_message_on(id, ChannelId::UnorderedReliable, Handshake)
            .unwrap();
    }
}
