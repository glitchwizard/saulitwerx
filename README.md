# Saulitwerx

An open-source, parametric 3D CAD modeler for mechanical design, written in Rust. Sketch, constrain, extrude, revolve, fillet, and edit the history later.

> **Status: pre-alpha.** Nothing here is usable for real work yet. See [docs/ROADMAP.md](docs/ROADMAP.md).

## Goals

- History-based part modeling: a feature tree you can edit and rebuild, driven by named parameters.
- A 2D sketcher with geometric and dimensional constraints, and degree-of-freedom feedback.
- Standard exchange formats: STEP and STL. Native documents are diffable text files.
- Runs on Linux, macOS and Windows, with a browser (WASM) build later.
- Permissively licensed, so anyone can use it, embed it or ship it.

## How it is built

This project is developed mostly by scheduled AI agent runs (a "dark factory") working from a public roadmap and issue queue. A human sets direction. The rules the agents follow are in [docs/factory/FACTORY.md](docs/factory/FACTORY.md). Contributions from humans are just as welcome. Please read [CLEAN_ROOM.md](CLEAN_ROOM.md) first.

## Building

```sh
cargo test --workspace
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option. Third-party components (notably the OpenCascade geometry kernel, LGPL-2.1 with exception) keep their own licenses; see [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md#licensing).

Saulitwerx is an independent project. It is not affiliated with, endorsed by, or derived from any commercial CAD product.
