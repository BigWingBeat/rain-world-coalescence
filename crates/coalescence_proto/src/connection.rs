use std::{io::Read, marker::PhantomData};

use ::serde::{de::DeserializeOwned, Serialize};
use bytes::Bytes;

use crate::{
    packet::{Packet, PacketSet},
    peer::Peer,
    serde::{deserialize_length_prefix, deserialize_packet_from, length_prefix_size, serialize},
    state::{ConnectionState, ConnectionStateObject},
    Error,
};

use self::byte_queue::ByteQueue;

mod byte_queue;

#[derive(Debug, Clone, Copy)]
pub enum Channel {
    /// Packets are guranteed to arrive in the same order they are sent
    Ordered,
    /// Packets are guranteed to arrive, but possibly in a different order than they were sent
    Unordered,
    /// Packets may be lost, and may arrive in a different order than they were sent
    Unreliable,
}

/// Some bytes to be transmitted to the peer via a certain channel
#[derive(Debug)]
pub struct Transmit {
    pub bytes: Bytes,
    pub channel: Channel,
}

/// A connection to a single peer.
///
/// The generic type parameter `P` is the type of *this* peer, not the remote peer.
#[derive(Debug)]
pub struct Connection<P> {
    state: Box<dyn ConnectionStateObject>,
    queued_state_change: Option<ConnectionState>,
    send_queue: Vec<Transmit>,
    receive_queue: ByteQueue,
    pending_receive_len: Option<usize>,
    peer: PhantomData<P>,
}

impl<P> Default for Connection<P> {
    fn default() -> Self {
        Self::new()
    }
}

impl<P> Connection<P> {
    pub fn new() -> Self {
        Self {
            state: crate::state::default_state(),
            queued_state_change: None,
            send_queue: Vec::new(),
            receive_queue: ByteQueue::new(),
            pending_receive_len: None,
            peer: PhantomData,
        }
    }

    pub fn state(&self) -> ConnectionState {
        self.state.state()
    }
}

impl<P: Peer> Connection<P> {
    /// Send the specified packet to the peer
    pub fn handle_send<T>(&mut self, packet: T) -> Result<(), Error>
    where
        T: Packet<P>,
    {
        let packet = packet.into_set();
        let bytes = serialize(&packet, self.state())?;

        // Do state handling after serialization so it doesn't happen if serialization errored
        self.state.handle_packet_serialize(&packet)?;
        if let Some(new_state) = self.state.poll_state_change() {
            self.queued_state_change = Some(new_state.state());
            self.state = new_state
        }

        // Only actually push the bytes to be transmitted if nothing errored
        if !bytes.is_empty() {
            self.send_queue.push(Transmit {
                bytes: bytes.into(),
                channel: T::CHANNEL,
            });
        }

        Ok(())
    }

    /// Handle some bytes that were received from the peer
    pub fn handle_receive(&mut self, received: Bytes) {
        self.receive_queue.push(received);
    }

    /// Poll for bytes to be sent to the peer
    pub fn poll_send(&mut self) -> Vec<Transmit> {
        std::mem::take(&mut self.send_queue)
    }

    /// Poll for packets received from the peer
    ///
    /// Returns Ok(Some(T)) if a packet was successfully received.
    ///
    /// Returns Ok(None) if no packets are currently available.
    ///
    /// If an error was encountered while receiving a packet, returns Err(_) and discards the packet
    pub fn poll_receive<T>(&mut self) -> Result<Option<T>, Error>
    where
        T: PacketSet,
    {
        let packet_length = match self.pending_receive_len.take() {
            Some(packet_length) => packet_length,
            None => {
                // Wait until we have enough bytes to decode the packet length
                if self.receive_queue.total_bytes() < length_prefix_size() {
                    return Ok(None);
                }

                let mut buf = [0, 0];
                self.receive_queue.read_exact(&mut buf).unwrap();
                deserialize_length_prefix(buf)
            }
        };

        // If we've decoded the packet length, but don't yet have the rest of the packet data, remember the length until we do
        if self.receive_queue.total_bytes() < packet_length {
            self.pending_receive_len = Some(packet_length);
            return Ok(None);
        }

        let prev_total_bytes = self.receive_queue.total_bytes();

        let state = self.state();

        // Use `.take()` to limit the bytes that can be read so in case the deserialization goes wrong it can't eat into the
        // data of subsequent packets
        match deserialize_packet_from((&mut self.receive_queue).take(packet_length as u64), state) {
            Ok(packet) => {
                assert_eq!(
                    prev_total_bytes - self.receive_queue.total_bytes(),
                    packet_length,
                    "Deserializing packet didn't read all of its bytes"
                );

                self.state.handle_packet_deserialize(&packet)?;
                if let Some(new_state) = self.state.poll_state_change() {
                    self.queued_state_change = Some(new_state.state());
                    self.state = new_state
                }

                Ok(Some(packet))
            }
            Err(e) => {
                // A deserialization error means this packet's data is most likely malformed and unusable, so we discard
                // any of it that has been left over from the deserialization potentially having halted prematurely
                let read_bytes = prev_total_bytes - self.receive_queue.total_bytes();
                let leftover_bytes = packet_length - read_bytes;
                self.receive_queue.discard_bytes(leftover_bytes);
                Err(e)
            }
        }
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

        client
            .handle_send(Profile {
                username: "asd".into(),
            })
            .unwrap();

        // Intended compile fails
        // client.handle_send(Lobby {
        //     usernames: Vec::new(),
        // });

        // client.handle_send(PlayerJoined { username: "asd" });
        // client.handle_send(PlayerLeft).unwrap();
    }

    #[test]
    fn client_de() {
        let mut client: Connection<Client> = Connection::new();

        // client.poll_receive::<HandshakePacket>().unwrap();
        // client.poll_receive::<LobbyPacket>().unwrap();
    }

    #[test]
    fn server_ser() {
        let mut server: Connection<Server> = Connection::new();

        server
            .handle_send(Lobby {
                usernames: Vec::new(),
            })
            .unwrap();

        server
            .handle_send(PlayerJoined {
                username: "asd".into(),
            })
            .unwrap();

        server.handle_send(PlayerLeft).unwrap();

        // Intended compile fails
        // server.handle_send(Profile { username: "asd" });
    }

    #[test]
    fn server_de() {
        let mut server: Connection<Server> = Connection::new();

        // server.poll_receive::<HandshakePacket>().unwrap();
        // server.poll_receive::<LobbyPacket>().unwrap();
    }
}
