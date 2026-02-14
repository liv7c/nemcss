//! LSP server implementation for NemCSS.
//!
//! Provides language server features including completions for utility classes and responsive
//! utilities.
mod cache;
mod context;
mod position;

use std::path::PathBuf;

use config::{CONFIG_FILE_NAME, NemCssConfig};
use dashmap::DashMap;
use miette::Diagnostic;
use ropey::Rope;
use thiserror::Error;
use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use crate::cache::NemCache;
use crate::position::lsp_col_to_byte;

#[derive(Debug)]
pub struct Backend {
    /// Client to interact with the LSP client.
    client: Client,
    /// Cache to store the generated utilities, viewports, custom properties, and content globs.
    cache: RwLock<Option<NemCache>>,
    /// Cache to keep track of open documents and their content.
    /// Rope is a wrapper around a String that provides helpers for working with text.
    documents: DashMap<String, Rope>,
    /// Workspace root directory
    workspace_root: RwLock<Option<PathBuf>>,
    /// Encoding used to calculate the character positions.
    /// It could be utf8 or utf16 (both should be supported)
    position_encoding: RwLock<PositionEncodingKind>,
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

        let position_encoding = params
            .capabilities
            .general
            .and_then(|general| general.position_encodings)
            .and_then(|encodings| {
                if encodings.contains(&PositionEncodingKind::UTF8) {
                    Some(PositionEncodingKind::UTF8)
                } else if encodings.contains(&PositionEncodingKind::UTF16) {
                    Some(PositionEncodingKind::UTF16)
                } else {
                    encodings.first().cloned()
                }
            })
            .unwrap_or(PositionEncodingKind::UTF16);

        *self.position_encoding.write().await = position_encoding.clone();

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                position_encoding: Some(position_encoding),
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
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        if let Err(e) = self.rebuild_cache().await {
            self.client
                .log_message(
                    MessageType::ERROR,
                    format!(
                        "failed to initialize NemCSS cache. Completions will not work: {}",
                        e
                    ),
                )
                .await;
        }

        if let Err(e) = self.setup_file_watchers().await {
            self.client
                .log_message(
                    MessageType::WARNING,
                    format!("failed to setup file watchers: {}", e),
                )
                .await;
        }

        self.client
            .log_message(MessageType::INFO, "nemcss server initialized with cache")
            .await;
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;

        self.documents.insert(uri.to_string(), Rope::from(text));
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;

        if let Some(content) = params.content_changes.into_iter().last() {
            self.documents
                .insert(uri.to_string(), Rope::from(content.text));
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.documents.remove(&params.text_document.uri.to_string());
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let position = &params.text_document_position.position;
        let uri = &params.text_document_position.text_document.uri;

        let rope_ref = match self.documents.get(&uri.to_string()) {
            Some(rope) => rope,
            None => return Ok(None),
        };

        let encoding = self.position_encoding.read().await;
        let line_idx = position.line as usize;
        let col = lsp_col_to_byte(&rope_ref, position, &encoding);

        let class_context = match context::detect_multiline_class_context(&rope_ref, line_idx, col)
        {
            Some(context) => context,
            None => return Ok(None),
        };

        let cache_guard = self.cache.read().await;
        let cache = match cache_guard.as_ref() {
            Some(cache) if cache.is_content_file(uri) => cache,
            _ => return Ok(None),
        };

        let partial = &class_context.partial_token;
        let mut completion_items: Vec<CompletionItem> = Vec::new();

        if class_context.responsive_prefix.is_some() {
            for responsive_utility in &cache.responsive_utilities {
                if partial.is_empty() || responsive_utility.responsive_class_name.contains(partial)
                {
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
        } else {
            for utility in &cache.utilities {
                if partial.is_empty() || utility.class_name().starts_with(partial) {
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
            }
        }

        Ok(Some(CompletionResponse::Array(completion_items)))
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        eprintln!("{:?}", params);

        Ok(Some(Hover {
            contents: HoverContents::Scalar(MarkedString::String("You're hovering!".to_string())),
            range: None,
        }))
    }

    async fn did_change_watched_files(&self, _: DidChangeWatchedFilesParams) {
        if let Err(e) = self.rebuild_cache().await {
            self.client
                .log_message(
                    MessageType::ERROR,
                    format!(
                        "failed to rebuild NemCSS cache after file change. Completions may be stale: {}",
                        e
                    ),
                )
                .await;
        }
    }
}

/// RebuildCacheError represents the error type when rebuilding the cache.
#[derive(Debug, Error, Diagnostic)]
enum RebuildCacheError {
    #[error("failed to get workspace root: {0}")]
    #[diagnostic(code(lsp_error::workspace_root_error))]
    WorkspaceRoot(std::io::Error),

    #[error(transparent)]
    #[diagnostic(code(lsp_error::cache_build_error))]
    CacheBuild(#[from] cache::BuildCacheError),
}

/// SetupFileWatchersError represents the error type when setting up file watchers.
#[derive(Debug, Error, Diagnostic)]
enum SetupFileWatchersError {
    #[error("failed to get workspace root")]
    #[diagnostic(code(lsp_error::workspace_root_error))]
    MissingWorkspaceRoot,

    #[error("tokens directory path contains invalid UTF-8")]
    #[diagnostic(code(lsp::setup_file_watchers::invalid_tokens_dir_path))]
    InvalidTokensDirPath,

    #[error("failed to read config: {0}")]
    #[diagnostic(code(lsp::setup_file_watchers::config_read))]
    ConfigRead(#[from] config::NemCssConfigError),

    #[error("failed to serialize registration options: {0}")]
    #[diagnostic(code(lsp_error::setup_file_watchers_error::serialize_registration_options))]
    SerializeRegistrationOptions(serde_json::Error),

    #[error("failed to register capability: {0}")]
    #[diagnostic(code(lsp_error::setup_file_watchers_error::register_capability))]
    RegisterCapability(tower_lsp::jsonrpc::Error),
}

impl Backend {
    /// Buids a new `Backend` instance.
    /// For the position encoding, we follow
    /// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocuments
    /// To stay backwards compatible the only mandatory encoding is UTF-16 represented via the string utf-16.
    /// The server can pick one of the encodings offered by the client and signals that encoding back to the client via the initialize result’s property capabilities.positionEncoding.
    pub fn new(client: Client) -> Self {
        Self {
            client,
            cache: RwLock::new(None),
            workspace_root: RwLock::new(None),
            documents: DashMap::new(),
            position_encoding: RwLock::new(PositionEncodingKind::UTF16),
        }
    }

    async fn rebuild_cache(&self) -> miette::Result<(), RebuildCacheError> {
        let workspace_root = self
            .workspace_root
            .read()
            .await
            .as_ref()
            .cloned()
            .or_else(|| std::env::current_dir().ok())
            .ok_or_else(|| {
                RebuildCacheError::WorkspaceRoot(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "workspace root not found",
                ))
            })?;
        let cache = NemCache::build(&workspace_root)?;

        self.cache.write().await.replace(cache);
        Ok(())
    }

    async fn setup_file_watchers(&self) -> miette::Result<(), SetupFileWatchersError> {
        let workspace_root = self
            .workspace_root
            .read()
            .await
            .as_ref()
            .cloned()
            .or_else(|| std::env::current_dir().ok())
            .ok_or(SetupFileWatchersError::MissingWorkspaceRoot)?;

        let config_path = workspace_root.join(CONFIG_FILE_NAME);
        let config = NemCssConfig::from_path(&config_path)?;

        // register file watchers for config and token file changes
        let tokens_dir_str = config
            .tokens_dir
            .to_str()
            .ok_or(SetupFileWatchersError::InvalidTokensDirPath)?;
        let tokens_glob_pattern = format!("**/{}/**/*.json", tokens_dir_str);
        let config_glob_pattern = format!("**/{}", CONFIG_FILE_NAME);

        let watchers = vec![
            FileSystemWatcher {
                glob_pattern: GlobPattern::String(tokens_glob_pattern),
                kind: Some(WatchKind::all()),
            },
            FileSystemWatcher {
                glob_pattern: GlobPattern::String(config_glob_pattern),
                kind: Some(WatchKind::all()),
            },
        ];

        let registration_options =
            serde_json::to_value(DidChangeWatchedFilesRegistrationOptions { watchers })
                .map_err(SetupFileWatchersError::SerializeRegistrationOptions)?;

        self.client
            .register_capability(vec![Registration {
                id: "nemcss-config-tokens-file-watchers".to_string(),
                method: "workspace/didChangeWatchedFiles".to_string(),
                register_options: Some(registration_options),
            }])
            .await
            .map_err(SetupFileWatchersError::RegisterCapability)?;

        Ok(())
    }
}
