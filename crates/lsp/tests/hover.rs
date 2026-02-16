mod common;

use common::{file_uri, fixture_path, init_context};
use insta::assert_snapshot;
use serde_json::json;

use crate::common::{LspNotification, LspRequest};

#[tokio::test]
async fn test_hover_shows_css_for_utility_class() {
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
                    //                ^ col 15 (on 'b' in "bg-black")
                    "character": 15,
                },
            }),
        )
        .await;

    let hover_text = result["contents"]["value"]
        .as_str()
        .expect("hover should return markdown");
    // Validate the formatting of the hover text
    assert_snapshot!(hover_text);
}

#[tokio::test]
async fn test_hover_shows_value_for_custom_property() {
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
                "text": ".foo { color: var(--color-white); }"
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
                    // ".foo { color: var(--color-white); }"
                    //                      ^ col 20
                    "character": 20,
                },
            }),
        )
        .await;

    let hover_text = result["contents"]["value"]
        .as_str()
        .expect("hover should return markdown");

    // Validate the formatting of the hover text
    assert_snapshot!(hover_text);
}

#[tokio::test]
async fn test_hover_returns_nothing_outside_class_attribute() {
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
                    //                                     ^ col 36
                    "character": 36,
                },
            }),
        )
        .await;

    assert!(
        result.is_null(),
        "hover should not return anything outside class attribute"
    );
}

#[tokio::test]
async fn test_hover_shows_css_for_responsive_class() {
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
                "text": "<div class=\"sm:bg-black text-white\"></div>"
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
                    // "<div class="sm:bg-black text-white"></div>"
                    //                 ^ col 15
                    "character": 15,
                },
            }),
        )
        .await;

    let hover_text = result["contents"]["value"]
        .as_str()
        .expect("hover should return markdown");
    // Validate the formatting of the hover text
    assert_snapshot!(hover_text);
}

#[tokio::test]
async fn test_hover_returns_nothing_for_unknown_class() {
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
                "text": "<div class=\"bg-nonexistent text-white\"></div>"
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
                    // "<div class="bg-nonexistent text-white"></div>"
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
}
