use std::{path::PathBuf, process::Stdio, str::FromStr};

use anyhow::{Context, Result};
use lsp_client::{transport::io_transport, LspClient};
use lsp_types::{notification::*, request::*, *};
use tokio::{process::Command, sync::oneshot};

#[tokio::main]
async fn main() -> Result<()> {
    let mut child = Command::new("rust-analyzer")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .context("failed to spawn rust-analyzer")?;

    let stdin = child.stdin.take().context("missing stdin")?;
    let stdout = child.stdout.take().context("missing stdout")?;
    let (tx, rx) = io_transport(stdin, stdout);
    let client = LspClient::new(tx, rx);

    // Channel to wait for rust-analyzer to finish indexing the workspace.
    let (indexed_tx, indexed_rx) = oneshot::channel();
    let mut subscription = client.subscribe_to_method::<Progress>().await?;
    tokio::spawn(async move {
        while let Some(msg) = subscription.next().await {
            let params = msg.unwrap();
            if matches!(params.token, NumberOrString::String(s) if s == "rustAnalyzer/Indexing")
                && matches!(
                    params.value,
                    ProgressParamsValue::WorkDone(WorkDoneProgress::End(_))
                )
            {
                indexed_tx.send(()).unwrap();
                break;
            }
        }
        subscription.unsubscribe().await.unwrap();
    });

    // Initialize rust-analyzer with the current file.
    let source_path = PathBuf::from(file!())
        .canonicalize()
        .context("failed to get current file path")?;
    let source_uri = Uri::from_str(format!("file://{}", source_path.display()).as_str())?;

    let initialize_params = InitializeParams {
        capabilities: ClientCapabilities {
            workspace: Some(WorkspaceClientCapabilities {
                workspace_folders: Some(true),
                ..Default::default()
            }),
            window: Some(WindowClientCapabilities {
                work_done_progress: Some(true),
                ..Default::default()
            }),
            ..Default::default()
        },
        workspace_folders: Some(vec![WorkspaceFolder {
            name: "root".to_string(),
            uri: source_uri.clone(),
        }]),
        ..Default::default()
    };
    client.initialize(initialize_params).await?;
    client.initialized().await?;

    println!("Initialized rust-analyzer");

    // Wait to finish indexing the workspace.
    indexed_rx
        .await
        .context("failed to receive indexing notification")?;

    println!("Finished indexing");

    let goto_definition_params = GotoDefinitionParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: source_uri },
            // Position of the `LspClient` import.
            position: lsp_types::Position {
                line: 3,
                character: 32,
            },
        },
        work_done_progress_params: Default::default(),
        partial_result_params: Default::default(),
    };
    let response = client
        .send_request::<GotoDefinition>(goto_definition_params)
        .await?
        .context("failed to get goto definition response")?;

    println!("Goto definition response: {response:?}");

    client.shutdown().await?;
    client.exit().await?;
    child.wait().await?;
    Ok(())
}
