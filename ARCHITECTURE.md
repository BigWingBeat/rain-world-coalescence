# Architecture

This file documents the layout of the repository, the architecture of the code, and the networking strategy used.

## The Repository

- WIP

## The Code

The server is written in pure Rust, with no CSharp code, and uses [Bevy ECS](https://bevyengine.org/) for the simulation logic. The client/mod is also written mostly in Rust for parity with the server, but has some CSharp code by necessity, in order to hook into and interface with the base game.

## The Networking

The mod uses authoritative replication with rollback and client-server (i.e. star) network. It is built on [Quinn](https://github.com/quinn-rs/quinn), an implementation of the [QUIC transport protocol](https://quicwg.org/).

## Useful links
- [This summary](https://github.com/bevyengine/bevy/discussions/8675) of the above networking terminology
- [Why use QUIC?](https://github.com/Henauxg/bevy_quinnet#quic-as-a-game-networking-protocol)
