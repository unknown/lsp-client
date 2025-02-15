# lsp-client

[![crates.io](https://img.shields.io/crates/v/lsp-client.svg)](https://crates.io/crates/lsp-client)
[![Documentation](https://docs.rs/lsp-client/badge.svg)](https://docs.rs/lsp-client)
[![MIT licensed](https://img.shields.io/crates/l/lsp-client.svg)](./LICENSE)

A client for the [Language Server Protocol](https://microsoft.github.io/language-server-protocol/).

## Usage

Start a language server and create a client to communicate with it.

```rust
let mut child = Command::new("rust-analyzer")
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .spawn()
    .unwrap();

let stdin = child.stdin.take().unwrap();
let stdout = child.stdout.take()..unwrap();
let (tx, rx) = io_transport(stdin, stdout);
let client = LspClient::new(tx, rx);
```

See the [examples](examples) directory for more usage examples.
