use std::marker::PhantomData;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    deserialize,
    packet::{IntoPacket, Packet},
    peer::Peer,
    serde::SerdeError,
    serialize,
    state::{ConnectionState, ConnectionStateHandler, PacketHandler, WrongStateError},
};

#[derive(Debug, Clone, Copy)]
pub enum Channel {
    /// Packets are guranteed to arrive in the same order they are sent
    Ordered,
    /// Packets are guranteed to arrive, but possibly in a different order than they were sent
    Unordered,
    /// Packets may be lost, and may arrive in a different order than they were sent
    Unreliable,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    WrongState(#[from] WrongStateError),
    #[error(transparent)]
    Serde(#[from] SerdeError),
}

/// A connection to a single peer
#[derive(Debug)]
pub struct Connection<P> {
    state: ConnectionStateHandler,
    queued_state_change: Option<ConnectionState>,
    peer: PhantomData<P>,
}

impl<P> Connection<P> {
    pub fn new() -> Self {
        Self {
            state: Default::default(),
            queued_state_change: None,
            peer: PhantomData,
        }
    }

    pub fn state(&self) -> ConnectionState {
        (&self.state).into()
    }
}

impl<P: Peer> Connection<P> {
    pub fn serialize<T>(&mut self, packet: impl IntoPacket<T, P>) -> Result<Vec<u8>, Error>
    where
        T: Packet + Serialize,
    {
        let handler = T::Handler::try_from_state(&mut self.state)?;
        let packet = packet.into_packet();
        let bytes = serialize(&packet)?;
        // Put handling after the all the ?s so it doesn't happen if there were any errors
        handler.handle_packet_serialize(&packet);
        if let Some(new_state) = handler.poll_state_change() {
            self.queued_state_change = Some((&new_state).into());
            self.state = new_state;
        }
        Ok(bytes)
    }

    pub fn deserialize<'de, T>(&mut self, bytes: &'de [u8]) -> Result<T, Error>
    where
        T: Packet + Deserialize<'de>,
    {
        let handler = T::Handler::try_from_state(&mut self.state)?;
        let packet = deserialize(bytes)?;
        handler.handle_packet_deserialize(&packet);
        if let Some(new_state) = handler.poll_state_change() {
            self.queued_state_change = Some((&new_state).into());
            self.state = new_state;
        }
        Ok(packet)
    }

    pub fn poll_state_change(&mut self) -> Option<ConnectionState> {
        self.queued_state_change.take()
    }
}

#[cfg(test)]
mod tests {
    use super::Connection;
    use crate::{
        packet::{HandshakePacket, Lobby, LobbyPacket, PlayerJoined, PlayerLeft, Profile},
        peer::{Client, Server},
    };

    #[test]
    fn client_ser() {
        let mut client: Connection<Client> = Connection::new();

        client.serialize(Profile { username: "asd" }).unwrap();

        // Intended compile fails
        // client.serialize(Lobby {
        //     usernames: Vec::new(),
        // });
        // client.serialize(PlayerJoined { username: "asd" });
        // client.serialize(PlayerLeft).unwrap();
    }

    #[test]
    fn client_de() {
        let mut client: Connection<Client> = Connection::new();
        let bytes = Vec::new();

        client.deserialize::<HandshakePacket>(&bytes).unwrap();
        client.deserialize::<LobbyPacket>(&bytes).unwrap();
    }

    #[test]
    fn server_ser() {
        let mut server: Connection<Server> = Connection::new();

        server
            .serialize(Lobby {
                usernames: Vec::new(),
            })
            .unwrap();
        server.serialize(PlayerJoined { username: "asd" }).unwrap();
        server.serialize(PlayerLeft).unwrap();

        // Intended compile fails
        // server.serialize(Profile { username: "asd" });
    }

    #[test]
    fn server_de() {
        let mut server: Connection<Server> = Connection::new();
        let bytes = Vec::new();

        server.deserialize::<HandshakePacket>(&bytes).unwrap();
        server.deserialize::<LobbyPacket>(&bytes).unwrap();
    }
}
