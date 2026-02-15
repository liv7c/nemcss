use std::path::{Path, PathBuf};

use lsp::Backend;
use serde_json::{Value, json};
use tower::{Service, ServiceExt};
use tower_lsp::{ClientSocket, LspService, jsonrpc::Request, lsp_types};

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

    pub async fn request(&mut self, method: &str, params: Value) -> Value {
        self.next_id += 1;

        let request = Request::build(method.to_string())
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
            None => Value::Null,
        }
    }

    /// Sends a notification to the server
    pub async fn notify(&mut self, method: &str, params: Value) {
        let notification = Request::build(method.to_string()).params(params).finish();

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
pub fn file_uri(path: &Path) -> String {
    lsp_types::Url::from_file_path(path)
        .expect("path should be absolute and valid")
        .to_string()
}

/// Creates a TestContext and run the initialize -> initialized handshake.
/// After this, the cache is built and the server is ready to receive requests.
pub async fn init_context(fixture: &str) -> TestContext {
    let mut ctx = TestContext::new();
    let root = fixture_path(fixture);
    let root_uri = file_uri(&root);

    let result = ctx
        .request(
            "initialize",
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

    ctx.notify("initialized", json!({})).await;
    ctx
}
