use bevy::{log::LogPlugin, prelude::*, tasks::IoTaskPool};
use multiplayer_mvp_net::AppEndpoint;
use quinn::{Connecting, Endpoint, ServerConfig};
use rcgen::RcgenError;
use rustls::{Certificate, PrivateKey};

fn main() {
    App::new()
        .add_plugins((LogPlugin::default(), MinimalPlugins))
        .add_systems(Startup, start_listening)
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

    let endpoint = multiplayer_mvp_net::server(
        server_config,
        multiplayer_mvp_net::IPV6_WILDCARD_DEFAULT_PORT,
    )?;

    Ok(endpoint)
}

fn start_listening(mut commands: Commands) {
    let endpoint = create_endpoint().unwrap();

    match endpoint.local_addr() {
        Ok(address) => info!("Server listening on '{address}'..."),
        Err(e) => error!("{}", e),
    }

    IoTaskPool::get()
        .spawn(accept_connections(endpoint.clone()))
        .detach();

    commands.insert_resource(AppEndpoint(endpoint));
}

async fn accept_connections(endpoint: Endpoint) {
    while let Some(connecting) = endpoint.accept().await {
        let address = connecting.remote_address();
        info!("Incoming connection from '{address}'...");
        IoTaskPool::get()
            .spawn(handle_connection(connecting))
            .detach();
    }
}

async fn handle_connection(connecting: Connecting) {
    if let Err(e) = try_handle_connection(connecting).await {
        error!("{}", e);
    }
}

async fn try_handle_connection(connecting: Connecting) -> anyhow::Result<()> {
    let connection = connecting.await?;
    let (mut send, mut receive) = connection.open_bi().await?;
    send.write_all("Server handshake".as_bytes()).await?;
    let mut read_buffer = [0u8; 128];
    if let Some(bytes_read) = receive.read(&mut read_buffer).await? {
        let data = &read_buffer[..bytes_read];
        let string = String::from_utf8_lossy(data);
        let address = connection.remote_address();
        let id = connection.stable_id();
        info!("Server got data from client '{address}' (ID {id}): '{string}'");
    }
    Ok(())
}
