use std::{net::SocketAddr, sync::Arc};

use quinn::{Endpoint, EndpointConfig, ServerConfig};
use runtime::BevyTasksRuntime;
use serde::{Deserialize, Serialize};

pub mod runtime;

pub const DEFAULT_PORT: u16 = 7110;

#[derive(Serialize, Deserialize)]
pub struct Handshake;

pub fn client(addr: SocketAddr) -> std::io::Result<Endpoint> {
    Endpoint::new(
        EndpointConfig::default(),
        None,
        std::net::UdpSocket::bind(addr)?,
        Arc::new(BevyTasksRuntime),
    )
}

pub fn server(config: ServerConfig, addr: SocketAddr) -> std::io::Result<Endpoint> {
    Endpoint::new(
        EndpointConfig::default(),
        Some(config),
        std::net::UdpSocket::bind(addr)?,
        Arc::new(BevyTasksRuntime),
    )
}
