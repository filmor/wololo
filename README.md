WoLolo
======

WoLolo is a simple webservice to send Wake-on-LAN magic packets to a device on
the local network.

## Development

WoLolo uses [Topcoat](https://github.com/tokio-rs/topcoat) for routing, views,
and asset serving. Install its CLI, then start the development server:

```sh
cargo install topcoat-cli
topcoat dev
```

`topcoat dev` builds the app, bundles `assets/pure-min.css`, and serves it on
`127.0.0.1:3000`. Use the existing `--host` and `--port` options with
`cargo run` after bundling assets manually:

```sh
cargo build
topcoat asset bundle --bin wololo
cargo run -- --host 0.0.0.0 --port 3000
```
