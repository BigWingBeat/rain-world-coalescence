# Architecture

This file documents the layout of the repository, the architecture of the code, and the networking strategy used.

## The Repository

The `mod` folder contains all of the CSharp code:
- `CoalescenceClient` is a mod for the [official Remix mod loader](https://store.steampowered.com/news/app/312520/view/3398546521169524646) for Rain World 1.9, which is based on [BepInEx](https://docs.bepinex.dev/). The mod uses [ClangSharp](https://github.com/dotnet/ClangSharp) to generate bindings to the native Rust code, which is where the majority of the simulation and networking is performed.

The `crates` folder contains all of the Rust code:
- The `coalescence_client` crate defines the C api that the CSharp code actually calls into. It is compiled as a [`cdylib`](https://doc.rust-lang.org/reference/linkage.html?highlight=cdylib) and uses [`cbindgen`](https://github.com/mozilla/cbindgen) to generate the C headers that are what ClangSharp actually uses as input to generate the CSharp bindings.
- `coalescence_server` is the dedicated server binary. It exclusively performs simulation and networking, and is functionally useless without a client to connect to it.
- `coalescence_proto` is a ["sans-IO"](https://sans-io.readthedocs.io/how-to-sans-io.html) implementation of all of the networking logic.
- `coalescence_quinn` integrates [Quinn](https://github.com/quinn-rs/quinn) with the protocol implementation to do actual I/O.

## The Code

The server is written in pure [Rust](https://www.rust-lang.org/), with no CSharp code, and uses [Bevy ECS](https://bevyengine.org/) for the simulation logic. The client/mod is also written mostly in Rust for parity with the server, but has some CSharp code by necessity, in order to hook into and interface with the base game.

## The Networking

The mod uses authoritative replication with rollback and client-server (i.e. star) network. It is built on [Quinn](https://github.com/quinn-rs/quinn), an implementation of the [QUIC transport protocol](https://quicwg.org/).

## Useful links
- [This summary](https://github.com/bevyengine/bevy/discussions/8675) of the above networking terminology
- [Why use QUIC?](https://github.com/Henauxg/bevy_quinnet#quic-as-a-game-networking-protocol)
