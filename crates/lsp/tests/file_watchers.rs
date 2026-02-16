mod common;

use insta::assert_snapshot;
use serde_json::json;

use crate::common::{
    LspNotification, LspRequest, copy_fixture_to_temp, file_uri, init_context_at_path,
};

#[tokio::test]
async fn test_hover_shows_css_for_utility_class_after_design_token_file_change() {
    let temp_dir = copy_fixture_to_temp("basic_project");
    let mut ctx = init_context_at_path(temp_dir.path()).await;

    let file_path = temp_dir.path().join("src").join("index.html");
    let uri = file_uri(&file_path);

    // Open a file using `bg-special` - this class does not exist in the design tokens file yet
    ctx.notify(
        LspNotification::DidOpen,
        json!({
            "textDocument": {
                "uri": uri,
                "languageId": "html",
                "version": 1,
                "text": "<div class=\"bg-special\"></div>"
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
                    // "<div class="bg-special"></div>"
                    //                ^ col 15
                    "character": 15,
                },
            }),
        )
        .await;

    assert!(
        result.is_null(),
        "hover should not return anything for unknown classes"
    );

    // Update the design tokens color file
    let colors_path = temp_dir.path().join("design-tokens").join("colors.json");
    std::fs::write(
        &colors_path,
        r#"
        {
            "title": "Color Tokens",
            "items": [{ "name": "special", "value": "hsl(0, 0%, 100%)" }]
        }
        "#,
    )
    .expect("failed to write colors design tokens file");

    let colors_uri = file_uri(&colors_path);
    ctx.notify(
        LspNotification::DidChangeWatchedFiles,
        json!({
            "changes": [{ "uri": colors_uri, "type" : 2 }]
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
                    // "<div class="bg-special"></div>"
                    //                ^ col 15
                    "character": 15,
                },
            }),
        )
        .await;

    let hover_text = result["contents"]["value"]
        .as_str()
        .expect("hover should return markdown");

    assert_snapshot!(hover_text);
}
