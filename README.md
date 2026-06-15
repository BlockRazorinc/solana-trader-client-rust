# Solana-trader-client-rust

Example for solana-trader-client in Rust.

# Document

See [document](https://blockrazor.gitbook.io/blockrazor/solana/send-transaction/rust).

# Transaction Encoding

| Encoding | gRPC Method | HTTP Endpoint | Request Field |
|---|---|---|---|
| Base64 | `SendTransaction` | `/sendTransaction` | `transaction` |
| Binary | `SendBinaryTransaction` | `/sendBinaryTransaction` | `binaryTransaction` |

For gRPC Binary requests, pass the serialized transaction `Vec<u8>` directly.
For HTTP Binary requests, convert the serialized transaction to Base64 because
the request body is JSON.

# Quickstart

Install [protoc](https://protobuf.dev/installation/#package-manager), then run:

```bash
git clone https://github.com/BlockRazorinc/solana-trader-client-rust.git
cd solana-trader-client-rust
cargo build --locked
```

# gRPC

| Mode | Encoding | Example |
|---|---|---|
| fast | Base64 | `cargo run --bin mode-grpc-fast` |
| fast | Binary | `cargo run --bin mode-grpc-fast-binary` |
| sandwichMitigation | Base64 | `cargo run --bin mode-grpc-sandwichMitigation` |
| sandwichMitigation | Binary | `cargo run --bin mode-grpc-sandwichMitigation-binary` |

# HTTP

| Mode | Encoding | Example |
|---|---|---|
| fast | Base64 | `cargo run --bin mode-http-fast` |
| fast | Binary | `cargo run --bin mode-http-fast-binary` |
| sandwichMitigation | Base64 | `cargo run --bin mode-http-sandwichMitigation` |
| sandwichMitigation | Binary | `cargo run --bin mode-http-sandwichMitigation-binary` |
