mod common;

use common::{TestContext, file_uri, fixture_path, init_context};
use insta::assert_json_snapshot;
use serde_json::json;

use crate::common::{LspNotification, LspRequest};

#[tokio::test]
async fn test_initialize_returns_capabilities() {
    let mut ctx = TestContext::new();
    let root = fixture_path("basic_project");
    let root_uri = file_uri(&root);

    let result = ctx
        .request(
            LspRequest::Initialize,
            json!({
                "processId": null,
                "rootUri": root_uri,
                "capabilities": {},
                "workspaceFolders": [{
                    "uri": root_uri,
                    "name": "test"
                }]
            }),
        )
        .await;

    // Check that the server returns the correct trigger characters + utf-16 encoding
    // if does not receive encoding information
    assert_json_snapshot!(result["capabilities"]);
}

#[tokio::test]
async fn test_did_close_remove_document() {
    let fixture = "basic_project";
    let mut ctx = init_context(fixture).await;
    let file_path = fixture_path(fixture).join("src").join("index.html");
    let uri = file_uri(&file_path);

    ctx.notify(
        LspNotification::DidOpen,
        json!({
            "textDocument": {
                "uri": uri,
                "languageId": "html",
                "version": 1,
                "text": "<div class=\"bg-black text-white\"></div>"
            }
        }),
    )
    .await;

    ctx.notify(
        LspNotification::DidClose,
        json!({
            "textDocument": {
                "uri": uri,
            }
        }),
    )
    .await;

    let result = ctx
        .request(
            LspRequest::Hover,
            json!({
                "textDocument": {
                    "uri": uri,
                },
                "position": {
                    "line": 0,
                    // "<div class="bg-black text-white"></div>"
                    //                ^ col 15
                    "character": 15,
                },
            }),
        )
        .await;

    assert!(
        result.is_null(),
        "closed documents should not return hover results"
    );
}
