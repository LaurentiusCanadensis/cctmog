
# cctmog — Seven-Twenty-Seven (7–27) with WebSockets + iced 0.13

## Build
```bash
cd cctmog
cargo build
```

## Run
Server:
```bash
cd server
cargo run -p cctmog-server
# listens on ws://0.0.0.0:9001/ws
```

Client (in another terminal; run multiple for more players):
```bash
cd ../client
cargo run -p cctmog
```

If you prefer `cargo run` without `-p`, `cd` into the specific package directory first.
