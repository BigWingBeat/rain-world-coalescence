# Rain World Multiplayer MVP (aka Coalescence)

This mod is an MVP (Minimum Viable Product) to demonstrate 'real' (Not via Parsec/Steam Remote Play etc.) online multiplayer for Rain World.

As an MVP, the scope of the mod is currently limited exclusively to the Sandbox and Competitive arena modes.

## Comparison with [Rain Meadow](https://github.com/henpemaz/Rain-Meadow)

Rain Meadow is another online-multiplayer mod for Rain World. I believe that Coalescence and Rain Meadow have different enough architectures and design goals to be able to co-exist alongside eachother.

This mod's architecture is detailed in the [ARCHITECTURE.md](./ARCHITECTURE.md) file in this repository. By comparison, Rain Meadow is written entirely in CSharp, and uses a peer-to-peer (i.e. mesh) network, which in turn means it does not have a dedicated server binary.

Rain Meadow is primarily focused on its namesake Meadow gamemode, which is inspired by the [game of the same name](https://store.steampowered.com/app/486310/Meadow/). Coalescence, by comparison, is primarily focused on an as-of-yet-unnamed gamemode which intends to turn Rain World into what SovietWomble coins an 'Ecosystem Survival' game in his [The Isle video essay](https://youtu.be/cj7JzmEf-_c).

## Compiling

Given that this is a mod for online multiplayer, it features both a client and a server that can be compiled and ran separately from eachother.

### Compiling the client/mod

As a prerequisite to compiling the mod, you need to install a few things:

- [The .NET SDK](https://learn.microsoft.com/en-us/dotnet/core/sdk) - for compiling the CSharp components
- [ClangSharp](https://github.com/dotnet/ClangSharp) - for generating bindings between the Rust and CSharp components
- [The Rust language toolchain](https://www.rust-lang.org/tools/install) - for compiling the Rust components
- And if you're not on Windows, [cross](https://github.com/cross-rs/cross) - for cross-compiling the Rust components

With those installed, you can compile the client by simply running `dotnet build -c Release` in the `mod/MultiplayerMvpClient` folder. This will compile and build the mod, placing it inside the `artifacts/bin/MultiplayerMvpClient/release_win-x86/mod` folder. You can use the compiled client by simply moving the built mod folder into your Rain World mods folder (`Rain World/RainWorld_Data/StreamingAssets/mods`) and enabling it from the in-game Remix menu.

### Compiling the server

The prerequisites for compiling the server are a subset of those for the client, so if you're already compiling the client, you don't need to install anything. Otherwise, you will need to install the following:

- [The Rust language toolchain](https://www.rust-lang.org/tools/install) - the server is written in pure Rust

To compile the server, you just need to run `cargo build --release --package multiplayer_mvp_server` in the root folder of the repository. The compiled server binary will be located somewhere in the `target` folder.
