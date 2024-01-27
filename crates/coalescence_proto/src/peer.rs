/// The side that initiated the connection, is the source of player inputs,
/// defers to the server in server-authoritative architectures
#[derive(Debug)]
pub enum Client {}

/// The side that accepts incoming connections, is the 'hub' in 'hub-and-spoke'-shaped networks,
/// is the single source of truth in server-authoritative architectures
#[derive(Debug)]
pub enum Server {}

/// One side of a connection between two devices
pub trait Peer: sealed::Sealed + Send + Sync + 'static {
    /// The device on the other side of the connection
    type Remote;
}

impl Peer for Client {
    type Remote = Server;
}

impl Peer for Server {
    type Remote = Client;
}

/// Data that is transmitted from the client to the server
#[derive(Debug)]
pub enum ClientToServer {}

/// Data that is transmitted from the server to the client
#[derive(Debug)]
pub enum ServerToClient {}

/// Data that can be transmitted from & to either peer
#[derive(Debug)]
pub enum Bidirectional {}

/// The direction that data is transmitted over the network between two peers
pub trait Direction: sealed::Sealed {}

impl Direction for ClientToServer {}
impl Direction for ServerToClient {}
impl Direction for Bidirectional {}

/// A direction that is outbound for the specified peer `P`, i.e. packets transmitted in this direction are serialized and sent
/// over the network, from the perspective of the specified peer
pub trait Outbound<P>: Direction {}

impl Outbound<Client> for ClientToServer {}
impl Outbound<Server> for ServerToClient {}

impl Outbound<Client> for Bidirectional {}
impl Outbound<Server> for Bidirectional {}

/// A direction that is inbound for the specified peer `P`, i.e. packets transmitted in this direction are received from the
/// network and deserialized, from the perspective of the specified peer
pub trait Inbound<P>: Direction {}

impl Inbound<Client> for ServerToClient {}
impl Inbound<Server> for ClientToServer {}

impl Inbound<Client> for Bidirectional {}
impl Inbound<Server> for Bidirectional {}

mod sealed {
    pub trait Sealed {}
    impl Sealed for super::Client {}
    impl Sealed for super::Server {}
    impl Sealed for super::ClientToServer {}
    impl Sealed for super::ServerToClient {}
    impl Sealed for super::Bidirectional {}
}
