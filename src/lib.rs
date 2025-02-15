pub mod transport;

use jsonrpsee::{
    async_client::{Client, ClientBuilder},
    core::{
        client::{
            ClientT, Subscription, SubscriptionClientT, TransportReceiverT, TransportSenderT,
        },
        traits::ToRpcParams,
    },
};
use lsp_types::{notification::*, request::*, *};
use serde::Serialize;
use serde_json::value::RawValue;

struct SerdeParam<T>(T)
where
    T: Serialize;

impl<T> ToRpcParams for SerdeParam<T>
where
    T: Serialize,
{
    fn to_rpc_params(self) -> Result<Option<Box<RawValue>>, serde_json::Error> {
        let json = serde_json::to_string(&self.0)?;
        RawValue::from_string(json).map(Some)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum LspError {
    #[error("jsonrpsee error: {0}")]
    Jsonrpsee(#[from] jsonrpsee::core::client::Error),
}

/// A client for the Language Server Protocol.
#[derive(Debug)]
pub struct LspClient {
    client: Client,
}

impl LspClient {
    pub fn new<S, R>(sender: S, receiver: R) -> Self
    where
        S: TransportSenderT + Send,
        R: TransportReceiverT + Send,
    {
        let client = ClientBuilder::default().build_with_tokio(sender, receiver);
        Self { client }
    }

    /// Request the server to initialize the client.
    ///
    /// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#initialize
    pub async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult, LspError> {
        self.send_request::<Initialize>(params).await
    }

    /// Notify the server that the client received the result of the `initialize` request.
    ///
    /// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#initialized
    pub async fn initialized(&self) -> Result<(), LspError> {
        let params = InitializedParams {};
        self.send_notification::<Initialized>(params).await
    }

    /// Request the server to shutdown the client.
    ///
    /// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#shutdown
    pub async fn shutdown(&self) -> Result<(), LspError> {
        self.send_request::<Shutdown>(()).await
    }

    /// Notify the server to exit the process.
    ///
    /// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#exit
    pub async fn exit(&self) -> Result<(), LspError> {
        self.send_notification::<Exit>(()).await
    }

    /// Send an LSP request to the server.
    pub async fn send_request<R>(&self, params: R::Params) -> Result<R::Result, LspError>
    where
        R: Request,
    {
        let result = self.client.request(R::METHOD, SerdeParam(params)).await?;
        Ok(result)
    }

    /// Send an LSP notification to the server.
    pub async fn send_notification<N>(&self, params: N::Params) -> Result<(), LspError>
    where
        N: Notification,
    {
        self.client
            .notification(N::METHOD, SerdeParam(params))
            .await?;
        Ok(())
    }

    /// Create a subscription to an LSP notification.
    pub async fn subscribe_to_method<N>(&self) -> Result<Subscription<N::Params>, LspError>
    where
        N: Notification,
    {
        let subscription = self.client.subscribe_to_method(N::METHOD).await?;
        Ok(subscription)
    }
}
