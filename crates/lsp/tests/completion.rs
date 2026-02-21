mod common;

use common::{file_uri, fixture_path, init_context};
use insta::assert_json_snapshot;
use serde_json::json;

use crate::common::{LspNotification, LspRequest};

fn completion_labels(result: &serde_json::Value) -> Vec<&str> {
    let items = result
        .as_array()
        .or_else(|| result["items"].as_array())
        .expect("completion response should be an array or have an 'items' field");

    let mut labels: Vec<&str> = items
        .iter()
        .map(|item| {
            item["label"]
                .as_str()
                .expect("completion item should have a 'label' field")
        })
        .collect();
    labels.sort();
    labels
}

#[tokio::test]
async fn test_completion_excluded_for_non_content_files() {
    let fixture = "basic_project";
    let mut ctx = init_context(fixture).await;
    let file_path = fixture_path(fixture).join("README.md");
    let uri = file_uri(&file_path);

    ctx.notify(
        LspNotification::DidOpen,
        json!({
            "textDocument": {
                "uri": uri,
                "languageId": "markdown",
                "version": 1,
                "text": "<div class=\"bg-\">"
            }
        }),
    )
    .await;

    let result = ctx
        .request(
            LspRequest::Completion,
            json!({
                "textDocument": {
                    "uri": uri,
                },
                "position": {
                    "line": 0,
                    // "<div class="bg-">"
                    //                ^ col 15
                    "character": 15,
                },
            }),
        )
        .await;

    assert!(
        result.is_null() || result.as_array().is_some_and(|arr| arr.is_empty()),
        "non-content files should not get class completions"
    );
}

#[tokio::test]
async fn test_did_change_updates_document_for_completion() {
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
                "text": "<div></div>"
            }
        }),
    )
    .await;

    ctx.notify(
        LspNotification::DidChange,
        json!({
            "textDocument": {
                "uri": uri,
                "version": 2
            },
            "contentChanges": [{
                "text": "<div class=\"text-\"></div>"
            }]
        }),
    )
    .await;

    let result = ctx
        .request(
            LspRequest::Completion,
            json!({
                "textDocument": {
                    "uri": uri,
                },
                "position": {
                    "line": 0,
                    // "<div class="text-"></div>"
                    //                  ^ col 17
                    "character": 17,
                },
            }),
        )
        .await;

    assert_json_snapshot!(completion_labels(&result));
}

#[tokio::test]
async fn test_completion_suggests_custom_properties_in_var_context() {
    let fixture = "basic_project";
    let mut ctx = init_context(fixture).await;
    let file_path = fixture_path(fixture).join("src").join("app.css");
    let uri = file_uri(&file_path);

    ctx.notify(
        LspNotification::DidOpen,
        json!({
            "textDocument": {
                "uri": uri,
                "languageId": "css",
                "version": 1,
                "text": ".foo { color: var(--) }"
            }
        }),
    )
    .await;

    let result = ctx
        .request(
            LspRequest::Completion,
            json!({
                "textDocument": {
                    "uri": uri,
                },
                "position": {
                    "line": 0,
                    // ".foo { color: var(--) }"
                    //                      ^ col 20
                    "character": 20,
                },
            }),
        )
        .await;

    assert_json_snapshot!(completion_labels(&result));
}

#[tokio::test]
async fn test_responsive_completion_has_text_edit_replacing_prefix() {
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
                "text": "<div class=\"md:\">"
            }
        }),
    )
    .await;

    let result = ctx
        .request(
            LspRequest::Completion,
            json!({
                "textDocument": {
                    "uri": uri,
                },
                "position": {
                    "line": 0,
                    // "<div class=\"md:\">"
                    //                ^ col 15
                    "character": 15,
                },
            }),
        )
        .await;

    let items = result.as_array().expect("should return an array");
    assert!(!items.is_empty(), "should return completion items");

    for item in items {
        let text_edit = &item["textEdit"];
        assert!(!text_edit.is_null(), "should have a text edit");

        let start_char = text_edit["range"]["start"]["character"]
            .as_u64()
            .expect("should have a start character");
        let end_char = text_edit["range"]["end"]["character"]
            .as_u64()
            .expect("should have an end character");
        let new_text = text_edit["newText"]
            .as_str()
            .expect("should have a new text");
        let label = item["label"].as_str().expect("should have a label");

        assert_eq!(start_char, 12, "range should start at the `m` of `md:`");

        assert_eq!(end_char, 15, "range should end at the end of cursor");

        assert_eq!(
            new_text, label,
            "text edit should replace `md:` with `md:[classname]`"
        );

        assert!(
            !label.starts_with("md:md:"),
            "label must not contain double prefix `md:md:`, got `{}`",
            label
        );
    }
}
