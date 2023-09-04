use std::{
    net::{SocketAddr, ToSocketAddrs},
    sync::Arc,
};

use anyhow::{bail, ensure};
use bevy::{
    app::{AppExit, ScheduleRunnerPlugin},
    ecs::event::ManualEventReader,
    log::LogPlugin,
    prelude::*,
};
use multiplayer_mvp_net::{AppEndpoint, NoServerVerification};
use quinn::{Connection, Endpoint};

#[derive(Debug)]
pub struct AppContainer {
    pub app: App,
    app_exit_event_reader: ManualEventReader<AppExit>,
}

impl AppContainer {
    pub fn new() -> std::io::Result<Self> {
        let mut app = App::new();

        // `create_endpoint` is called inline so that it happens after the `TaskPoolPlugin` has initialised
        // the default task pools in `add_plugins` (Plugins get eagerly built as soon as they're added).
        app.add_plugins((
            LogPlugin::default(),
            MinimalPlugins.build().disable::<ScheduleRunnerPlugin>(),
        ))
        .insert_resource(AppEndpoint(create_endpoint()?));
        // .add_systems(Startup, create_endpoint);

        while !app.ready() {
            bevy::tasks::tick_global_task_pools_on_main_thread();
        }
        app.finish();
        app.cleanup();

        Ok(Self {
            app,
            app_exit_event_reader: default(),
        })
    }

    pub fn update(&mut self) -> Option<AppExit> {
        self.app.update();

        self.app
            .world
            .get_resource::<Events<AppExit>>()
            .and_then(|app_exit_events| {
                self.app_exit_event_reader
                    .iter(app_exit_events)
                    .last()
                    .cloned()
            })
    }

    pub async fn connect_to_server(
        endpoint: &Endpoint,
        address: &str,
        port: u16,
    ) -> anyhow::Result<()> {
        let mut addresses = (address, port).to_socket_addrs()?;
        let addresses = addresses.as_mut_slice();
        let len = addresses.len();

        ensure!(
            len > 0,
            "Could not connect to '{address}:{port}' as it did not resolve to any addresses"
        );

        info!("Resolved '{address}:{port}' to {len} addresses.");

        // let endpoint = self.app.world.resource::<AppEndpoint>();
        let mut connection = None;

        // `SocketAddr` implements `Ord` such that IPv4 addresses get sorted before IPv6 addresses, so we sort the
        // given addresses and then iterate over them in reverse, meaning IPv6 addresses get prioritised.
        addresses.sort_unstable();
        for &address in addresses.iter().rev() {
            info!("Connecting to '{address}'...");

            match try_connect(endpoint, address).await {
                Ok(c) => {
                    connection = Some(c);
                    break;
                }
                Err(e) => error!("Could not connect: {e}"),
            };

            // Use a local function for the `?` syntax, as `try` blocks are unstable
            async fn try_connect(
                endpoint: &Endpoint,
                address: SocketAddr,
            ) -> anyhow::Result<Connection> {
                let connection = endpoint.connect(address, &address.to_string())?.await?;
                Ok(connection)
            }
        }

        let Some(connection) = connection else {
            bail!("Could not connect to any of the addresses that '{address}:{port}' resolved to");
        };

        let (mut send, mut receive) = connection.accept_bi().await?;

        let mut read_buffer = [0u8; 128];
        if let Some(bytes_read) = receive.read(&mut read_buffer).await? {
            let data = &read_buffer[..bytes_read];
            let string = String::from_utf8_lossy(data);
            let address = connection.remote_address();
            let id = connection.stable_id();
            info!("Client got data from server '{address}' (ID {id}): '{string}'");
        }

        send.write_all("Client handshake".as_bytes()).await?;

        Ok(())
    }
}

fn create_config() -> quinn::ClientConfig {
    // Exactly the same as `with_safe_defaults()` but with TLS 1.2 disabled (Quic requires TLS 1.3)
    let mut crypto = rustls::ClientConfig::builder()
        .with_safe_default_cipher_suites()
        .with_safe_default_kx_groups()
        .with_protocol_versions(&[&rustls::version::TLS13])
        .unwrap()
        .with_custom_certificate_verifier(Arc::new(NoServerVerification))
        .with_no_client_auth();
    crypto.enable_early_data = true;

    quinn::ClientConfig::new(Arc::new(crypto))
}

fn create_endpoint() -> std::io::Result<Endpoint> {
    let mut endpoint = multiplayer_mvp_net::client(multiplayer_mvp_net::IPV6_WILDCARD)?;
    let config = create_config();
    endpoint.set_default_client_config(config);
    Ok(endpoint)
}

// #[no_mangle]
// extern "C" fn create_app() -> Box<AppContainer> {
//     Box::new(AppContainer::new())
// }

// #[no_mangle]
// extern "C" fn update_app(app: Option<&mut AppContainer>) {
//     let Some(app) = app else { return };

//     if app.update().is_some() {
//         println!("App requested exit");
//     }
// }

// #[no_mangle]
// extern "C" fn drop_app(_: Option<Box<AppContainer>>) {}
