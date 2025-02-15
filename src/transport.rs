use jsonrpsee::core::client::{ReceivedMessage, TransportReceiverT, TransportSenderT};
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader};

/// Error that can occur when reading or sending messages on a transport.
#[derive(thiserror::Error, Debug)]
pub enum TransportError {
    /// Error in I/O operation.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    /// Error in parsing message.
    #[error("parse error: {0}")]
    Parse(String),
}

/// Sending end of I/O transport.
pub struct Sender<T>(T)
where
    T: AsyncWrite + Send + Unpin + 'static;

#[async_trait::async_trait]
impl<T> TransportSenderT for Sender<T>
where
    T: AsyncWrite + Send + Unpin + 'static,
{
    type Error = TransportError;

    async fn send(&mut self, msg: String) -> Result<(), Self::Error> {
        let msg_with_header = format!("Content-Length: {}\r\n\r\n{}", msg.len(), msg);
        self.0.write_all(msg_with_header.as_bytes()).await?;
        Ok(())
    }
}

/// Receiving end of I/O transport.
pub struct Receiver<T>(BufReader<T>)
where
    T: AsyncRead + Send + Unpin + 'static;

#[async_trait::async_trait]
impl<T> TransportReceiverT for Receiver<T>
where
    T: AsyncRead + Send + Unpin + 'static,
{
    type Error = TransportError;

    async fn receive(&mut self) -> Result<ReceivedMessage, Self::Error> {
        let mut content_length: Option<usize> = None;

        // Parse header part.
        // https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#baseProtocol
        let mut line = String::new();
        loop {
            self.0.read_line(&mut line).await?;
            match line.as_str() {
                // End of header.
                "\r\n" => break,
                // Content-Length: the length of the content part in bytes.
                line if line.starts_with("Content-Length: ") => {
                    // "Content-Length: " is 16 chars long and the last 2 chars are \r\n.
                    let len = &line[16..line.len() - 2];
                    let len = len
                        .parse::<usize>()
                        .map_err(|e| TransportError::Parse(e.to_string()))?;
                    content_length = Some(len);
                }
                _ => {}
            }
            line.clear();
        }

        let content_length = content_length.ok_or(TransportError::Parse(
            "Content-Length header not found".to_string(),
        ))?;
        let mut buf = vec![0; content_length];
        self.0.read_exact(&mut buf).await?;
        Ok(ReceivedMessage::Bytes(buf))
    }
}

/// Create a I/O transport `Sender` and `Receiver` pair.
pub fn io_transport<I, O>(input: I, output: O) -> (Sender<I>, Receiver<O>)
where
    I: AsyncWrite + Send + Unpin + 'static,
    O: AsyncRead + Send + Unpin + 'static,
{
    let sender = Sender(input);
    let receiver = Receiver(BufReader::new(output));
    (sender, receiver)
}
