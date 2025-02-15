# lsp-client

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
