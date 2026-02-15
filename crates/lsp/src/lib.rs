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

        // Check for var(--...) context
        let current_line = rope_ref.line(line_idx).to_string();
        if let Some(var_ctx) = context::detect_var_context(&current_line, col) {
            let cache_guard = self.cache.read().await;
            let cache = match cache_guard.as_ref() {
                Some(cache) if cache.is_relevant_file(uri) => cache,
                _ => return Ok(None),
            };

            let partial_property = &var_ctx.partial_property;
            let items: Vec<CompletionItem> = cache
                .custom_properties
                .iter()
                .filter(|prop| {
                    partial_property.is_empty() || prop.name.starts_with(partial_property)
                })
                .map(|prop| CompletionItem {
                    label: prop.name.to_string(),
                    kind: Some(CompletionItemKind::PROPERTY),
                    documentation: Some(Documentation::MarkupContent(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: format!("```css\n{}: {};\n```", prop.name, prop.value),
                    })),
                    ..Default::default()
                })
                .collect();

            return Ok(Some(CompletionResponse::Array(items)));
        }

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
        let uri = &params.text_document_position_params.text_document.uri;
        let position = &params.text_document_position_params.position;

        let rope_ref = match self.documents.get(&uri.to_string()) {
            Some(rope) => rope,
            None => return Ok(None),
        };

        let encoding = self.position_encoding.read().await;
        let line_idx = position.line as usize;
        let col = lsp_col_to_byte(&rope_ref, position, &encoding);
        let line_str = rope_ref.line(line_idx).to_string();

        if let Some(prop_name) = context::extract_var_property(&line_str, col) {
            let cache_guard = self.cache.read().await;
            let cache = match cache_guard.as_ref() {
                Some(cache) if cache.is_relevant_file(uri) => cache,
                _ => return Ok(None),
            };

            let prop = cache
                .custom_properties
                .iter()
                .find(|prop| prop.name == prop_name);
            return Ok(prop.map(|prop| Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!("```css\n{}: {};\n```", prop.name, prop.value),
                }),
                range: None,
            }));
        }

        let (content, col, span) = match context::find_class_span(&line_str, col) {
            Some(span) => (line_str, col, span),
            None => {
                let (combined, combined_col) = context::build_multiline_window(
                    &rope_ref,
                    line_idx,
                    col,
                    context::MAX_SCAN_LINES,
                );
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

        let css = cache
            .utilities
            .iter()
            .find(|u| u.class_name() == token)
            .map(|u| u.full_class().to_string())
            .or_else(|| {
                cache
                    .responsive_utilities
                    .iter()
                    .find(|u| u.responsive_class_name == token)
                    .map(|u| u.full_css_definition.to_string())
            });

        Ok(css.map(|definition| Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: format!("```css\n{}\n```", definition),
            }),
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
