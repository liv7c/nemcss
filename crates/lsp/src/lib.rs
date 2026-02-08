mod cache;

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
    pub client: Client,
    cache: RwLock<Option<NemCache>>,
    /// Cache to keep track of open documents and their content.
    documents: DashMap<String, String>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
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

        if let Some(cache) = self.cache.read().await.as_ref()
            && !cache.is_content_file(uri)
        {
            return Ok(None);
        }

        eprintln!("==================");
        Ok(Some(CompletionResponse::Array(vec![
            CompletionItem::new_simple("Hello".to_string(), "Some detail".to_string()),
            CompletionItem::new_simple("Bye".to_string(), "More detail".to_string()),
        ])))
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
            documents: DashMap::new(),
        }
    }

    async fn rebuild_cache(&self) -> miette::Result<(), LspError> {
        let workspace_root = std::env::current_dir().map_err(LspError::WorkspaceRoot)?;
        let cache = NemCache::build(&workspace_root)?;

        self.cache.write().await.replace(cache);
        Ok(())
    }
}
