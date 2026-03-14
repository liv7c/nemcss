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
async fn test_semantic_utility_appears_in_html_completions() {
    let fixture = "semantic_project";
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
                "text": "<div class=\"text-\"></div>"
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
                    // "<div class=\"text-\">"
                    //                   ^ col 17
                    "character": 17,
                },
            }),
        )
        .await;

    let labels = completion_labels(&result);
    assert!(
        labels.contains(&"text-primary"),
        "should contain semantic utility 'text-primary', got: {:?}",
        labels
    );
    assert!(
        labels.contains(&"text-secondary"),
        "should contain semantic utility 'text-secondary', got: {:?}",
        labels
    );
}

#[tokio::test]
async fn test_completion_suggests_token_refs_when_editing_semantic_config() {
    let fixture = "semantic_project";
    let mut ctx = init_context(fixture).await;
    let file_path = fixture_path(fixture).join("nemcss.config.json");
    let uri = file_uri(&file_path);

    ctx.notify(
        LspNotification::DidOpen,
        json!({
            "textDocument": {
                "uri": uri,
                "languageId": "json",
                "version": 1,
                // Simulate typing a new semantic token value referencing a primitive token
                "text": "{ \"tertiary\": \"{colors.\" }"
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
                    // "{ "tertiary": "{colors." }"
                    //                        ^ col 23 (after "{colors.")
                    "character": 23,
                },
            }),
        )
        .await;

    let labels = completion_labels(&result);
    assert!(
        !labels.is_empty(),
        "should return token reference completions"
    );
    assert!(
        labels.iter().all(|l| l.starts_with("{colors.")),
        "all completions should be colors token references, got: {:?}",
        labels
    );
    assert!(
        labels.contains(&"{colors.white}"),
        "should contain '{{colors.white}}', got: {:?}",
        labels
    );
    assert!(
        labels.contains(&"{colors.black}"),
        "should contain '{{colors.black}}', got: {:?}",
        labels
    );
}

#[tokio::test]
async fn test_config_token_ref_completion_has_text_edit_replacing_brace_and_partial() {
    let fixture = "semantic_project";
    let mut ctx = init_context(fixture).await;
    let file_path = fixture_path(fixture).join("nemcss.config.json");
    let uri = file_uri(&file_path);

    ctx.notify(
        LspNotification::DidOpen,
        json!({
            "textDocument": {
                "uri": uri,
                "languageId": "json",
                "version": 1,
                // Cursor is after `{colors.`
                "text": "{ \"tertiary\": \"{colors.\" }"
            }
        }),
    )
    .await;

    let result = ctx
        .request(
            LspRequest::Completion,
            json!({
                "textDocument": { "uri": uri },
                "position": {
                    "line": 0,
                    // "{ "tertiary": "{colors." }"
                    //                        ^ col 23
                    "character": 23,
                },
            }),
        )
        .await;

    let items = result.as_array().expect("should return an array");
    assert!(!items.is_empty(), "should return completion items");

    for item in items {
        let text_edit = item
            .get("textEdit")
            .expect("completion item should have a textEdit");
        let range = text_edit
            .get("range")
            .expect("textEdit should have a range");

        let start_char = range
            .get("start")
            .and_then(|s| s.get("character"))
            .and_then(|c| c.as_u64())
            .expect("should have a start character");
        let end_char = range
            .get("end")
            .and_then(|e| e.get("character"))
            .and_then(|c| c.as_u64())
            .expect("should have an end character");
        let new_text = text_edit
            .get("newText")
            .and_then(|t| t.as_str())
            .expect("should have newText");
        let label = item
            .get("label")
            .and_then(|l| l.as_str())
            .expect("should have a label");

        // `{colors.` is 8 chars; range should start at the `{` (col 15)
        assert_eq!(
            start_char, 15,
            "range should start at the opening `{{` of the token reference"
        );
        assert_eq!(end_char, 23, "range should end at the cursor");
        assert_eq!(new_text, label, "newText should equal the label");
        assert!(
            label.starts_with('{') && label.ends_with('}'),
            "label should be wrapped in braces, got `{}`",
            label
        );
    }
}

#[tokio::test]
async fn test_config_token_ref_completion_text_edit_consumes_auto_inserted_closing_brace() {
    // Simulates Neovim's auto-bracket insertion: when the user types `{`,
    // the editor inserts `{}` and places the cursor between the braces.
    // After typing a partial, the buffer looks like `"{colors.wh}"` with the
    // cursor before the closing `}`. The text_edit must cover that `}` so
    // the accepted completion doesn't leave a stray `}` behind.
    let fixture = "semantic_project";
    let mut ctx = init_context(fixture).await;
    let file_path = fixture_path(fixture).join("nemcss.config.json");
    let uri = file_uri(&file_path);

    ctx.notify(
        LspNotification::DidOpen,
        json!({
            "textDocument": {
                "uri": uri,
                "languageId": "json",
                "version": 1,
                // `}` already present right after the partial (auto-inserted by editor)
                "text": "{ \"tertiary\": \"{colors.}\" }"
            }
        }),
    )
    .await;

    let result = ctx
        .request(
            LspRequest::Completion,
            json!({
                "textDocument": { "uri": uri },
                "position": {
                    "line": 0,
                    // "{ "tertiary": "{colors.}" }"
                    //                        ^ col 23 (cursor before `}`)
                    "character": 23,
                },
            }),
        )
        .await;

    let items = result.as_array().expect("should return an array");
    assert!(!items.is_empty(), "should return completion items");

    for item in items {
        let text_edit = item
            .get("textEdit")
            .expect("completion item should have a textEdit");
        let range = text_edit
            .get("range")
            .expect("textEdit should have a range");

        let end_char = range
            .get("end")
            .and_then(|e| e.get("character"))
            .and_then(|c| c.as_u64())
            .expect("should have an end character");

        // Range end must be 24 (one past the `}`) so the auto-inserted brace is consumed
        assert_eq!(
            end_char, 24,
            "range end should extend past the auto-inserted `}}` (col 24)"
        );
    }
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

#[tokio::test]
async fn test_completion_suggests_semantic_properties_at_declaration_position() {
    let fixture = "semantic_project";
    let mut ctx = init_context(fixture).await;
    let file_path = fixture_path(fixture).join("src").join("index.css");
    let uri = file_uri(&file_path);

    ctx.notify(
        LspNotification::DidOpen,
        json!({
            "textDocument": {
                "uri": uri,
                "languageId": "css",
                "version": 1,
                "text": "body[data-theme=\"dark\"] {\n  --\n}"
            }
        }),
    )
    .await;

    let result = ctx
        .request(
            LspRequest::Completion,
            json!({
                "textDocument": { "uri": uri },
                "position": {
                    "line": 1,
                    "character": 4,
                },
            }),
        )
        .await;

    let labels = completion_labels(&result);
    assert!(
        labels.contains(&"--text-primary"),
        "should contain semantic property '--text-primary', got: {:?}",
        labels
    );
    assert!(
        labels.contains(&"--text-secondary"),
        "should contain semantic property '--text-secondary', got: {:?}",
        labels
    );
    assert!(
        !labels.contains(&"--color-white"),
        "should not suggest primitive '--color-white', got: {:?}",
        labels
    );
}

#[tokio::test]
async fn test_completion_filters_semantic_properties_by_partial_name() {
    let fixture = "semantic_project";
    let mut ctx = init_context(fixture).await;
    let file_path = fixture_path(fixture).join("src").join("index.css");
    let uri = file_uri(&file_path);

    ctx.notify(
        LspNotification::DidOpen,
        json!({
            "textDocument": {
                "uri": uri,
                "languageId": "css",
                "version": 1,
                "text": "body { --text-p }"
            }
        }),
    )
    .await;

    let result = ctx
        .request(
            LspRequest::Completion,
            json!({
                "textDocument": { "uri": uri },
                "position": {
                    "line": 0,
                    "character": 15,
                },
            }),
        )
        .await;

    let labels = completion_labels(&result);
    assert!(
        labels.contains(&"--text-primary"),
        "should contain semantic property '--text-primary', got: {:?}",
        labels
    );
    assert!(
        !labels.contains(&"--text-secondary"),
        "should not suggest semantic property '--text-secondary', got: {:?}",
        labels
    );
}

#[tokio::test]
async fn test_completion_fires_inside_html_style_block() {
    let fixture = "semantic_project";
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
                "text": "<html><head><style>\nbody {\n  --\n}\n</style></head><body></body></html>"
            }
        }),
    )
    .await;

    let result = ctx
        .request(
            LspRequest::Completion,
            json!({
                "textDocument": { "uri": uri },
                "position": {
                    "line": 2,
                    "character": 4,
                },
            }),
        )
        .await;

    let labels = completion_labels(&result);
    assert!(
        labels.contains(&"--text-primary"),
        "should contain semantic property '--text-primary', got: {:?}",
        labels
    );
    assert!(
        labels.contains(&"--text-secondary"),
        "should contain semantic property '--text-secondary', got: {:?}",
        labels
    );
}

#[tokio::test]
async fn test_completion_does_not_fire_outside_html_style_block() {
    let fixture = "semantic_project";
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
                "text": "<html><body>\n  --\n</body></html>"
            }
        }),
    )
    .await;

    let result = ctx
        .request(
            LspRequest::Completion,
            json!({
                "textDocument": { "uri": uri },
                "position": {
                    "line": 1,
                    "character": 4,
                },
            }),
        )
        .await;

    assert!(
        result.is_null() || result.as_array().is_some_and(|arr| arr.is_empty()),
        "should not return completions outside of style blocks"
    );
}
