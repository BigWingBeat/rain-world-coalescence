use std::{
    fs::File,
    io,
    net::{SocketAddr, ToSocketAddrs},
    sync::Arc,
};

use anyhow::anyhow;
use bevy::{
    app::{AppExit, PluginsState, ScheduleRunnerPlugin},
    ecs::event::ManualEventReader,
    log::Level,
    prelude::*,
    tasks::{block_on, IoTaskPool, Task},
};
use coalescence_common::{
    receive_stream_driver::ReceiveStreamDriver, send_stream_driver::SendStreamDriver, AppEndpoint,
    NoServerVerification,
};
use futures_lite::future::poll_once;
use quinn::{ConnectError, Connection, ConnectionError, Endpoint};
use thiserror::Error;
use tracing_log::LogTracer;
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, EnvFilter, Registry};

#[derive(Debug, Deref, DerefMut)]
pub struct AppContainer {
    #[deref]
    pub app: App,
    app_exit_event_reader: ManualEventReader<AppExit>,
}

impl Default for AppContainer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Resource, Debug)]
struct ServerConnection {
    username: String,
    connection: Connection,
    send: SendStreamDriver,
    receive: ReceiveStreamDriver,
}

#[derive(Debug, Error)]
pub enum ConnectToServerError {
    #[error("Could not create a QUIC endpoint")]
    CouldNotCreateEndpoint(#[source] io::Error),
    #[error("Could not resolve a socket address")]
    BadSocketAddress(#[source] io::Error),
    #[error(transparent)]
    ConnectError(#[from] ConnectError),
    #[error(transparent)]
    ConnectionError(#[from] ConnectionError),
    #[error(
        "Could not connect to any of the resolved socket addresses. See log output for details."
    )]
    AllAddressesFailed,
}

// Non-send resource because of the CSharp callbacks
#[derive(Debug)]
struct ConnectToServerTask {
    task: Task<Result<ServerConnection, ConnectToServerError>>,
    ok_handler: extern "C" fn(),
    error_handler: extern "C" fn(anyhow::Error),
}

impl AppContainer {
    pub fn new() -> Self {
        info!("AppContainer::new()");

        let mut app = App::new();
        app.add_plugins(MinimalPlugins.build().disable::<ScheduleRunnerPlugin>())
            .add_systems(
                Update,
                (
                    poll_connect_to_server_task,
                    handshake.run_if(resource_added::<ServerConnection>()),
                    drive_send_stream.run_if(resource_exists::<ServerConnection>()),
                ),
            );

        if app.plugins_state() != PluginsState::Cleaned {
            while app.plugins_state() == PluginsState::Adding {
                bevy::tasks::tick_global_task_pools_on_main_thread();
            }
            app.finish();
            app.cleanup();
        }

        Self {
            app,
            app_exit_event_reader: default(),
        }
    }

    pub fn update(&mut self) -> Option<AppExit> {
        self.app.update();

        self.app
            .world
            .get_resource::<Events<AppExit>>()
            .and_then(|app_exit_events| {
                self.app_exit_event_reader
                    .read(app_exit_events)
                    .last()
                    .cloned()
            })
    }

    pub fn connect_to_server(
        &mut self,
        address: &str,
        port: u16,
        username: String,
        async_ok_handler: extern "C" fn(),
        async_error_handler: extern "C" fn(anyhow::Error),
    ) -> Result<(), ConnectToServerError> {
        let address_port = format!("'{address}:{port}'");
        info!("Connecting to {address_port} with username '{username}'...");

        // Getting the endpoint cannot happen in the task because of needing to insert it as a resource if it's created here
        let endpoint = match self.world.get_resource::<AppEndpoint>() {
            Some(AppEndpoint(endpoint)) => endpoint.clone(),
            None => {
                let endpoint =
                    create_endpoint().map_err(ConnectToServerError::CouldNotCreateEndpoint)?;
                self.insert_resource(AppEndpoint(endpoint.clone()));
                endpoint
            }
        };

        // `to_socket_addrs` is blocking with no async alternative, so putting it in the task makes no difference
        let mut addresses = (address, port)
            .to_socket_addrs()
            .map_err(ConnectToServerError::BadSocketAddress)?;

        self.app.insert_non_send_resource(ConnectToServerTask {
            task: IoTaskPool::get().spawn(async move {
                let addresses = addresses.as_mut_slice();

                if addresses.is_empty() {
                    return Err(ConnectToServerError::BadSocketAddress(io::Error::new(
                        io::ErrorKind::Other,
                        format!("{address_port} resolved to 0 socket addresses"),
                    )));
                }

                info!(
                    "Resolved {address_port} to {} socket addresses.",
                    addresses.len()
                );

                let mut connection = None;

                // `SocketAddr` implements `Ord` such that IPv4 addresses get sorted before IPv6 addresses, so we sort the
                // given addresses and then iterate over them in reverse, meaning IPv6 addresses get prioritised.
                addresses.sort_unstable();
                for (i, &address) in addresses.iter().rev().enumerate() {
                    info!("Trying to connect to address #{}: '{address}'...", i + 1);

                    match try_connect(&endpoint, address).await {
                        Ok(c) => {
                            connection = Some(c);
                            break;
                        }
                        Err(e) => error!("{e}"),
                    };

                    // Use a local function for the `?` syntax, as `try` blocks are unstable
                    async fn try_connect(
                        endpoint: &Endpoint,
                        address: SocketAddr,
                    ) -> Result<Connection, ConnectToServerError> {
                        // The server_name parameter must either be a valid DNS domain name or a valid IpAddr, with the port excluded
                        Ok(endpoint
                            .connect(address, &address.ip().to_string())?
                            .await?)
                    }
                }

                let Some(connection) = connection else {
                    return Err(ConnectToServerError::AllAddressesFailed);
                };

                let (send, receive) = connection.open_bi().await?;

                info!("Connection established!");

                Ok(ServerConnection {
                    username,
                    connection,
                    send: SendStreamDriver::new(send),
                    receive: ReceiveStreamDriver::new(receive),
                })
            }),
            ok_handler: async_ok_handler,
            error_handler: async_error_handler,
        });

        Ok(())
    }
}

// Needs to be an exclusive system to be able to remove the non-send ConnectToServerTask resource
fn poll_connect_to_server_task(world: &mut World) {
    if let Some(mut task) = world.get_non_send_resource_mut::<ConnectToServerTask>() {
        if let Some(result) = block_on(poll_once(&mut task.task)) {
            match result {
                Ok(connection) => {
                    (task.ok_handler)();
                    world.insert_resource(connection);
                }
                // Only anyhow errors are allowed to cross the FFI boundry for simplicity
                Err(e) => (task.error_handler)(anyhow!(e)),
            }
            world.remove_non_send_resource::<ConnectToServerTask>();
        }
    }
}

fn drive_send_stream(mut connection: ResMut<ServerConnection>) {
    if let Err(e) = connection.send.drive() {
        error!("Error while sending data to server: {e}");
    }
}

fn handshake(mut connection: ResMut<ServerConnection>) {
    info!("Initiating handshake...");
    let bytes = connection.username.clone().into();
    connection.send.queue_chunk(bytes);
}

/// Configures native logging permanently for the whole application. Calling this more than once will panic.
/// This is used rather than Bevy's built-in `LogPlugin`, because that plugin configures logging in a way we
/// don't want, and that isn't configurable.
pub fn configure_logging() {
    const DEFAULT_LOG_LEVEL: Level = Level::INFO;
    const DEFAULT_LOG_FILTER: &str = "wgpu=error,naga=warn";

    let log_file = File::create(concat!(env!("CARGO_PKG_NAME"), ".native.log")).unwrap();

    let subscriber = Registry::default()
        .with(
            EnvFilter::try_from_default_env()
                .or_else(|_| {
                    EnvFilter::try_new(format!("{},{}", DEFAULT_LOG_LEVEL, DEFAULT_LOG_FILTER))
                })
                .unwrap(),
        )
        .with(
            tracing_subscriber::fmt::Layer::default()
                .with_ansi(false)
                .with_writer(log_file),
        );

    LogTracer::init().unwrap();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let old_handler = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |infos| {
        // Adapted from the std library's default panic handler
        let location = infos.location().unwrap();

        let msg = match infos.payload().downcast_ref::<&'static str>() {
            Some(s) => *s,
            None => match infos.payload().downcast_ref::<String>() {
                Some(s) => &s[..],
                None => "Box<dyn Any>",
            },
        };
        let thread = std::thread::current();
        let name = thread.name().unwrap_or("<unnamed>");

        error!("thread '{name}' panicked at {location}:\n{msg}");
        error!(
            "stack backtrace:\n{}",
            std::backtrace::Backtrace::force_capture()
        );
        old_handler(infos);
    }));
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

pub fn create_endpoint() -> std::io::Result<Endpoint> {
    let mut endpoint = coalescence_common::client(coalescence_common::IPV6_WILDCARD)?;
    let config = create_config();
    endpoint.set_default_client_config(config);
    Ok(endpoint)
}
