use std::{borrow::Cow, time::Duration};

use bevy::{app::ScheduleRunnerPlugin, log::LogPlugin, prelude::*, tasks::IoTaskPool};
use coalescence_common::{
    receive_stream_driver::ReceiveStreamDriver, send_stream_driver::SendStreamDriver, AppEndpoint,
};
use crossbeam::channel::{Receiver, Sender};
use quinn::{Connecting, Connection, Endpoint, ServerConfig};
use rcgen::RcgenError;
use rustls::{Certificate, PrivateKey};

#[derive(Debug, Default)]
enum ClientHandshakeState {
    #[default]
    ExpectingUsername,
    Finished(ClientProfile),
}

#[derive(Debug)]
struct ClientProfile {
    pub username: String,
}

#[derive(Component, Debug)]
struct ClientConnection {
    connection: Connection,
    handshake: ClientHandshakeState,
    send: SendStreamDriver,
    receive: ReceiveStreamDriver,
}

#[derive(Resource, Debug)]
struct NewClientConnectionReceiver(Receiver<ClientConnection>);

fn main() {
    App::new()
        .add_plugins((
            LogPlugin::default(),
            MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
                1.0 / 60.0, // Run server at 60 updates/s
            ))),
        ))
        .add_systems(Startup, start_listening)
        .add_systems(
            Update,
            (poll_new_client_connections, drive_send_streams, handshake),
        )
        .run();
}

fn generate_certificate(
    alt_names: impl Into<Vec<String>>,
) -> Result<(Certificate, PrivateKey), RcgenError> {
    let certificate = rcgen::generate_simple_self_signed(alt_names)?;
    Ok((
        Certificate(certificate.serialize_der()?),
        PrivateKey(certificate.serialize_private_key_der()),
    ))
}

fn create_endpoint() -> anyhow::Result<Endpoint> {
    let (certificate, private_key) = generate_certificate(vec!["::1".into()])?;

    let server_config = ServerConfig::with_single_cert(vec![certificate], private_key)?;
    // let transport_config = Arc::get_mut(&mut server_config.transport).unwrap();
    // transport_config.max_concurrent_uni_streams(0_u8.into());

    let endpoint = coalescence_common::server(
        server_config,
        coalescence_common::IPV6_WILDCARD_DEFAULT_PORT,
    )?;

    Ok(endpoint)
}

fn start_listening(mut commands: Commands) {
    let (sender, receiver) = crossbeam::channel::bounded(16);
    commands.insert_resource(NewClientConnectionReceiver(receiver));

    let endpoint = create_endpoint().unwrap();

    match endpoint.local_addr() {
        Ok(address) => info!("Server listening on '{address}'..."),
        Err(e) => error!("{}", e),
    }

    IoTaskPool::get()
        .spawn(accept_connections(endpoint.clone(), sender))
        .detach();

    commands.insert_resource(AppEndpoint(endpoint));
}

async fn accept_connections(endpoint: Endpoint, sender: Sender<ClientConnection>) {
    while let Some(connecting) = endpoint.accept().await {
        let address = connecting.remote_address();
        if let Some(local_ip) = connecting.local_ip() {
            info!("Incoming connection from '{address}' with local IP '{local_ip}'...");
        } else {
            info!("Incoming connection from '{address}'...");
        }
        IoTaskPool::get()
            .spawn(handle_connection(connecting, sender.clone()))
            .detach();
    }
}

async fn handle_connection(connecting: Connecting, sender: Sender<ClientConnection>) {
    let address = connecting.remote_address();
    let local_ip = connecting.local_ip();
    if let Err(e) = try_handle_connection(connecting, sender).await {
        if let Some(local_ip) = local_ip {
            error!("Error while handling incoming connection from '{address}' with local IP '{local_ip}': {e}");
        } else {
            error!("Error while handling incoming connection from '{address}': {e}");
        }
    }
}

async fn try_handle_connection(
    connecting: Connecting,
    sender: Sender<ClientConnection>,
) -> anyhow::Result<()> {
    let connection = connecting.await?;
    let (send, receive) = connection.accept_bi().await?;

    let client_connection = ClientConnection {
        connection,
        handshake: default(),
        send: SendStreamDriver::new(send),
        receive: ReceiveStreamDriver::new(receive),
    };

    sender.send(client_connection)?;
    Ok(())
}

fn poll_new_client_connections(mut commands: Commands, receiver: Res<NewClientConnectionReceiver>) {
    for new_connection in receiver.0.try_iter() {
        let id = new_connection.connection.stable_id();
        let address = new_connection.connection.remote_address();
        if let Some(local_ip) = new_connection.connection.local_ip() {
            info!("Connection established with client ID '{id}', address '{address}' and local IP '{local_ip}'!");
        } else {
            info!("Connection established with client ID '{id}', address '{address}'!");
        }
        commands.spawn(new_connection);
    }
}

fn drive_send_streams(mut query: Query<&mut ClientConnection>) {
    for mut client in query.iter_mut() {
        if let Err(e) = client.send.drive() {
            error!(
                "Error while sending data to client ID '{}': {e}",
                client.connection.stable_id()
            );
        }
    }
}

fn handshake(mut query: Query<&mut ClientConnection>) {
    for mut client in query.iter_mut() {
        match client.handshake {
            ClientHandshakeState::ExpectingUsername => {
                match client.receive.try_receive(256, true) {
                    Some(Ok(Some(username))) => {
                        let username = String::from_utf8_lossy(&username.bytes);
                        let warn = matches!(username, Cow::Owned(_));
                        let username = username.into_owned();

                        if warn {
                            warn!(
                                "Received non-UTF8 username from client ID '{}': '{username}'",
                                client.connection.stable_id()
                            );
                        } else {
                            info!(
                                "Received username from client ID '{}': '{username}'",
                                client.connection.stable_id()
                            );
                        }

                        client.handshake =
                            ClientHandshakeState::Finished(ClientProfile { username });
                    }
                    Some(Err(e)) => error!(
                        "Error while receiving username from client ID '{}': {e}",
                        client.connection.stable_id()
                    ),
                    Some(Ok(None)) | None => continue,
                }
            }
            ClientHandshakeState::Finished(_) => continue,
        }
    }
}
