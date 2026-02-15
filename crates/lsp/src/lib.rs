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
                        // Var prefix
                        "-".to_string(),
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

        // Spawn file watcher setup in the background so the `initialized` handler
        // returns immediately. `register_capability` sends a server→client request
        // and awaits a response, which would deadlock in test environments where
        // nobody drives the `ClientSocket`.
        let client = self.client.clone();
        let workspace_root = self.workspace_root.read().await.clone();
        tokio::spawn(async move {
            if let Err(e) = Backend::do_setup_file_watchers(client.clone(), workspace_root).await {
                client
                    .log_message(
                        MessageType::WARNING,
                        format!("failed to setup file watchers: {}", e),
                    )
                    .await;
            }
        });

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

        let ResolvedCursorPosition {
            rope,
            col,
            line_idx,
        } = match self.resolve_cursor_position(uri, position).await {
            Some(resolved_cursor_position) => resolved_cursor_position,
            None => return Ok(None),
        };

        // Check for var(--...) context
        let current_line = rope.line(line_idx).to_string();
        if let Some(var_ctx) = context::detect_var_context(&current_line, col) {
            let cache_guard = self.cache.read().await;
            let cache = match cache_guard.as_ref() {
                Some(cache) if cache.is_relevant_file(uri) => cache,
                _ => return Ok(None),
            };

            let partial_property = &var_ctx.partial_property;
            let items: Vec<CompletionItem> = cache.var_completions(partial_property);

            return Ok(Some(CompletionResponse::Array(items)));
        }

        let class_context = match context::detect_multiline_class_context(&rope, line_idx, col) {
            Some(context) => context,
            None => return Ok(None),
        };

        let cache_guard = self.cache.read().await;
        let cache = match cache_guard.as_ref() {
            Some(cache) if cache.is_content_file(uri) => cache,
            _ => return Ok(None),
        };

        let partial = &class_context.partial_token;

        let completion_items = if class_context.responsive_prefix.is_some() {
            cache.responsive_class_completions(partial)
        } else {
            cache.class_completions(partial)
        };

        Ok(Some(CompletionResponse::Array(completion_items)))
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = &params.text_document_position_params.position;

        let ResolvedCursorPosition {
            rope,
            col,
            line_idx,
        } = match self.resolve_cursor_position(uri, position).await {
            Some(resolved_cursor_position) => resolved_cursor_position,
            None => return Ok(None),
        };

        let line_str = rope.line(line_idx).to_string();

        if let Some(prop_name) = context::extract_var_property(&line_str, col) {
            let cache_guard = self.cache.read().await;
            let cache = match cache_guard.as_ref() {
                Some(cache) if cache.is_relevant_file(uri) => cache,
                _ => return Ok(None),
            };

            return Ok(cache.hover_for_custom_property(&prop_name));
        }

        let (content, col, span) = match context::find_class_span(&line_str, col) {
            Some(span) => (line_str, col, span),
            None => {
                let (combined, combined_col) =
                    context::build_multiline_window(&rope, line_idx, col, context::MAX_SCAN_LINES);
                match context::find_class_span(&combined, combined_col) {
                    Some(span) => (combined, combined_col, span),
                    None => return Ok(None),
                }
            }
        };

        let (span_start, span_end) = span;

        let token = match context::extract_token_at_cursor(&content, span_start, col, span_end) {
            Some(token) => token,
            None => return Ok(None),
        };

        let cache_guard = self.cache.read().await;
        let cache = match cache_guard.as_ref() {
            Some(cache) if cache.is_content_file(uri) => cache,
            _ => return Ok(None),
        };

        Ok(cache.hover_for_class(&token))
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
    #[error("workspace root is not set and current directory is unavailable")]
    #[diagnostic(code(lsp_error::workspace_root_error))]
    WorkspaceRoot,

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

/// ResolvedCursorPosition represents the resolved cursor position
/// from a given URI and LSP position.
/// It handles the conversion from LSP position to byte offset (for documents that are UTF-16).
#[derive(Debug, PartialEq)]
struct ResolvedCursorPosition {
    /// The document rope
    rope: Rope,
    /// The byte offset of the cursor
    col: usize,
    /// The line index of the cursor
    line_idx: usize,
}

impl Backend {
    /// Builds a new `Backend` instance.
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

    /// Resolves the document rope and cursor byte offset from a given URI and LSP position.
    /// Returns `None` if the document is not currently open.
    async fn resolve_cursor_position(
        &self,
        uri: &Url,
        position: &Position,
    ) -> Option<ResolvedCursorPosition> {
        let rope = self.documents.get(&uri.to_string())?.value().clone();

        let encoding = self.position_encoding.read().await;
        let line_idx = position.line as usize;
        let col = lsp_col_to_byte(&rope, position, &encoding);

        Some(ResolvedCursorPosition {
            rope,
            col,
            line_idx,
        })
    }

    async fn rebuild_cache(&self) -> miette::Result<(), RebuildCacheError> {
        let workspace_root = self
            .workspace_root
            .read()
            .await
            .as_ref()
            .cloned()
            .or_else(|| std::env::current_dir().ok())
            .ok_or(RebuildCacheError::WorkspaceRoot)?;
        let cache = NemCache::build(&workspace_root)?;

        self.cache.write().await.replace(cache);
        Ok(())
    }

    async fn do_setup_file_watchers(
        client: Client,
        workspace_root: Option<PathBuf>,
    ) -> miette::Result<(), SetupFileWatchersError> {
        let workspace_root = workspace_root
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

        client
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
