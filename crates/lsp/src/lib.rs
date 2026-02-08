mod cache;

use std::path::PathBuf;

use dashmap::DashMap;
use miette::Diagnostic;
use thiserror::Error;
use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use crate::cache::NemCache;

#[derive(Debug)]
pub struct Backend {
    /// Client to interact with the LSP client.
    client: Client,
    /// Cache to store the generated utilities, viewports, custom properties, and content globs.
    cache: RwLock<Option<NemCache>>,
    /// Cache to keep track of open documents and their content.
    documents: DashMap<String, String>,
    /// Workspace root directory
    workspace_root: RwLock<Option<PathBuf>>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        let workspace_root = params
            .workspace_folders
            .and_then(|folders| folders.into_iter().next())
            .and_then(|folder| folder.uri.to_file_path().ok())
            .or_else(|| params.root_uri.and_then(|uri| uri.to_file_path().ok()));

        if let Some(workspace_root) = workspace_root {
            self.workspace_root.write().await.replace(workspace_root);
        }

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![
                        // space between classes
                        " ".to_string(),
                        // Opening double quote
                        "\"".to_string(),
                        // Opening single quote
                        "'".to_string(),
                        // Responsive prefix
                        ":".to_string(),
                    ]),
                    ..CompletionOptions::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        if let Err(e) = self.rebuild_cache().await {
            self.client
                .log_message(MessageType::ERROR, miette::Report::new(e))
                .await;
        }

        self.client
            .log_message(MessageType::INFO, "nemcss server initialized with cache")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = &params.text_document_position.text_document.uri;
        eprintln!("=== COMPLETION ===");
        eprintln!("{:#?}", params);

        let mut completion_items: Vec<CompletionItem> = vec![];

        if let Some(cache) = self.cache.read().await.as_ref() {
            if !cache.is_content_file(uri) {
                return Ok(None);
            }

            for utility in cache.utilities.iter() {
                let documentation_markdown = format!("```css\n{}\n```", utility.full_class());

                completion_items.push(CompletionItem {
                    label: utility.class_name().to_string(),
                    kind: Some(CompletionItemKind::VALUE),
                    documentation: Some(tower_lsp::lsp_types::Documentation::MarkupContent(
                        MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: documentation_markdown,
                        },
                    )),
                    ..Default::default()
                });
            }

            for responsive_utility in cache.responsive_utilities.iter() {
                let documentation_markdown =
                    format!("```css\n{}\n```", responsive_utility.full_css_definition);

                completion_items.push(CompletionItem {
                    label: responsive_utility.responsive_class_name.to_string(),
                    kind: Some(CompletionItemKind::VALUE),
                    documentation: Some(tower_lsp::lsp_types::Documentation::MarkupContent(
                        MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: documentation_markdown,
                        },
                    )),
                    ..Default::default()
                })
            }
        }

        eprintln!("=============");
        eprintln!("{:#?}", completion_items);
        eprintln!("=============");

        Ok(Some(CompletionResponse::Array(completion_items)))
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        eprintln!("=== HOVER ===");
        eprintln!("{:#?}", params);
        eprintln!("=============");
        Ok(Some(Hover {
            contents: HoverContents::Scalar(MarkedString::String("You're hovering!".to_string())),
            range: None,
        }))
    }
}

#[derive(Debug, Error, Diagnostic)]
enum LspError {
    #[error("failed to get workspace root: {0}")]
    #[diagnostic(code(lsp_error::workspace_root_error))]
    WorkspaceRoot(std::io::Error),

    #[error(transparent)]
    #[diagnostic(code(lsp_error::cache_build_error))]
    CacheBuild(#[from] cache::BuildCacheError),
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            cache: RwLock::new(None),
            workspace_root: RwLock::new(None),
            documents: DashMap::new(),
        }
    }

    async fn rebuild_cache(&self) -> miette::Result<(), LspError> {
        let workspace_root = self
            .workspace_root
            .read()
            .await
            .as_ref()
            .cloned()
            .or_else(|| std::env::current_dir().ok())
            .ok_or_else(|| {
                LspError::WorkspaceRoot(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "workspace root not found",
                ))
            })?;
        let cache = NemCache::build(&workspace_root)?;

        self.cache.write().await.replace(cache);
        Ok(())
    }
}
