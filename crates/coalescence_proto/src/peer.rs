#[derive(Debug)]
pub struct Client;

#[derive(Debug)]
pub struct Server;

pub trait Peer: sealed::Sealed {}

impl Peer for Client {}
impl Peer for Server {}

mod sealed {
    pub trait Sealed {}
    impl Sealed for super::Client {}
    impl Sealed for super::Server {}
}
