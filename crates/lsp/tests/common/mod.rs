use std::path::{Path, PathBuf};

use assert_fs::TempDir;
use assert_fs::prelude::*;
use lsp::Backend;
use serde_json::{Value, json};
use tower::{Service, ServiceExt};
use tower_lsp::{ClientSocket, LspService, jsonrpc::Request, lsp_types};

/// Contains the different LSP notifications that we expect to receive
#[allow(dead_code)]
pub enum LspNotification {
    Initialized,
    DidOpen,
    DidChange,
    DidClose,
    DidChangeWatchedFiles,
}

impl LspNotification {
    fn method(&self) -> &'static str {
        match self {
            LspNotification::Initialized => "initialized",
            LspNotification::DidOpen => "textDocument/didOpen",
            LspNotification::DidChange => "textDocument/didChange",
            LspNotification::DidClose => "textDocument/didClose",
            LspNotification::DidChangeWatchedFiles => "workspace/didChangeWatchedFiles",
        }
    }
}

/// Contains the different LSP requests that we expect to receive
#[allow(dead_code)]
pub enum LspRequest {
    Initialize,
    Completion,
    Hover,
}

impl LspRequest {
    fn method(&self) -> &'static str {
        match self {
            LspRequest::Initialize => "initialize",
            LspRequest::Completion => "textDocument/completion",
            LspRequest::Hover => "textDocument/hover",
        }
    }
}

/// A test wrapper that drives the LspService directly as a tower service.
pub struct TestContext {
    pub service: LspService<Backend>,
    /// Socket for server->client communication
    /// We hold it to keep the channel alive
    _socket: ClientSocket,
    next_id: i64,
}

impl Default for TestContext {
    fn default() -> Self {
        Self::new()
    }
}

impl TestContext {
    pub fn new() -> Self {
        // Initialize the service with our backend
        let (service, socket) = LspService::new(Backend::new);
        TestContext {
            service,
            _socket: socket,
            next_id: 0,
        }
    }

    pub async fn request(&mut self, request: LspRequest, params: Value) -> Value {
        self.next_id += 1;

        let request = Request::build(request.method().to_string())
            .id(self.next_id)
            .params(params)
            .finish();

        let response = self
            .service
            .ready()
            .await
            .expect("service not ready")
            .call(request)
            .await
            .expect("service call failed");

        match response {
            Some(resp) => {
                let (_id, result) = resp.into_parts();
                match result {
                    Ok(val) => val,
                    Err(err) => panic!("LSP error: {:?}", err),
                }
            }
            None => panic!("request produced no response"),
        }
    }

    /// Sends a notification to the server
    pub async fn notify(&mut self, notification: LspNotification, params: Value) {
        let notification = Request::build(notification.method().to_string())
            .params(params)
            .finish();

        let response = self
            .service
            .ready()
            .await
            .expect("service not ready")
            .call(notification)
            .await
            .expect("service call failed");

        assert!(
            response.is_none(),
            "notification should not produce a response"
        );
    }
}

/// Returns the path to the fixture project
pub fn fixture_path(fixture: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(fixture)
}

/// Converts a path to a file URI
pub fn file_uri(path: &Path) -> lsp_types::Url {
    lsp_types::Url::from_file_path(path).expect("path should be absolute and valid")
}

/// Copies a fixture directory to a temporary directory
#[allow(dead_code)]
pub fn copy_fixture_to_temp(fixture: &str) -> TempDir {
    let src = fixture_path(fixture);
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    temp_dir
        .copy_from(&src, &["**/*"])
        .expect("failed to copy fixture");
    temp_dir
}

/// Creates a TestContext and run the initialize -> initialized handshake.
/// After this, the cache is built and the server is ready to receive requests.
#[allow(dead_code)]
pub async fn init_context(fixture: &str) -> TestContext {
    let mut ctx = TestContext::new();
    let root = fixture_path(fixture);
    let root_uri = file_uri(&root);

    let result = ctx
        .request(
            LspRequest::Initialize,
            json!({
                "processId": null,
                "rootUri": root_uri,
                "capabilities": {
                    "general": {
                        "positionEncodings": ["utf-8"],
                    }
                },
                "workspaceFolders": [{
                    "uri": root_uri,
                    "name": "test"
                }]
            }),
        )
        .await;

    assert!(result["capabilities"]["completionProvider"].is_object());
    assert!(
        result["capabilities"]["hoverProvider"]
            .as_bool()
            .unwrap_or(false)
    );

    ctx.notify(LspNotification::Initialized, json!({})).await;
    ctx
}

/// Creates a TestContext pointing at an arbitrary path and run the initialize -> initialized handshake.
#[allow(dead_code)]
pub async fn init_context_at_path(path: &Path) -> TestContext {
    let mut ctx = TestContext::new();
    let root_uri = file_uri(path);

    let result = ctx
        .request(
            LspRequest::Initialize,
            json!({
                "processId": null,
                "rootUri": root_uri,
                "capabilities": {
                    "general": {
                        "positionEncodings": ["utf-8"],
                    }
                },
                "workspaceFolders": [{
                    "uri": root_uri,
                    "name": "test"
                }]
            }),
        )
        .await;

    assert!(result["capabilities"]["completionProvider"].is_object());
    assert!(
        result["capabilities"]["hoverProvider"]
            .as_bool()
            .unwrap_or(false)
    );

    ctx.notify(LspNotification::Initialized, json!({})).await;
    ctx
}
