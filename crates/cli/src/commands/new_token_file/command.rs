use std::{fs, path::Path};

use config::{CONFIG_FILE_NAME, NemCssConfig, TokenFile};
use miette::Result;
use owo_colors::OwoColorize;
use serde_json::json;

use crate::commands::new_token_file::{
    error::NewTokenFileError,
    scale::{ScaleSource, build_items},
};

/// Registers a token file under `theme.<name>` in the nemcss configuration file,
/// preserving the existing formatting of all other keys.
fn register_in_config(
    config_path: &Path,
    name: &str,
    prefix: &str,
    source: &str,
    force: bool,
) -> Result<(), NewTokenFileError> {
    let content = fs::read_to_string(config_path).map_err(NewTokenFileError::ReadConfigFile)?;
    let mut config: serde_json::Value =
        serde_json::from_str(&content).map_err(NewTokenFileError::ParseConfigFile)?;

    let root = config
        .as_object_mut()
        .ok_or(NewTokenFileError::ConfigNotAnObject)?;

    let theme = root
        .entry("theme")
        .or_insert_with(|| json!({}))
        .as_object_mut()
        .ok_or(NewTokenFileError::ThemeNotAnObject)?;

    if theme.contains_key(name) && !force {
        return Err(NewTokenFileError::ThemeEntryExists {
            name: name.to_string(),
        });
    }

    theme.insert(
        name.to_string(),
        json!({"prefix": prefix, "source": source }),
    );

    // validate the the patched config is valid before overwriting the current config file
    serde_json::from_value::<config::NemCssConfig>(config.clone())
        .map_err(NewTokenFileError::PatchedConfigInvalid)?;

    let output = serde_json::to_string_pretty(&config).map_err(NewTokenFileError::Serialize)?;

    fs::write(config_path, output + "\n").map_err(NewTokenFileError::WriteConfigFile)?;

    Ok(())
}

/// Everything the `new-token-file` token needs to create a token file,
/// regardless of whether the options were collected via flags or from interactive prompts.
pub struct TokenFileRequest {
    pub name: String,
    pub source: ScaleSource,
    pub names: Option<Vec<String>>,
    pub unit: String,
    pub prefix: String,
}

/// Command to generate a new token file
pub fn new_token_file(request: TokenFileRequest, force: bool) -> Result<(), NewTokenFileError> {
    let TokenFileRequest {
        name,
        source,
        names,
        unit,
        prefix,
    } = request;
    let current_dir = std::env::current_dir().map_err(NewTokenFileError::RetrieveCurrentDir)?;

    let config_path = current_dir.join(CONFIG_FILE_NAME);
    if !config_path.exists() {
        return Err(NewTokenFileError::ConfigFileNotFound { path: config_path });
    }

    let config = NemCssConfig::from_path(&config_path).map_err(NewTokenFileError::LoadConfig)?;

    let items = build_items(&source, names.as_deref(), &unit)?;

    let tokens_dir = current_dir.join(&config.tokens_dir);
    fs::create_dir_all(&tokens_dir).map_err(NewTokenFileError::CreateTokensDir)?;

    let token_file_path = tokens_dir.join(format!("{name}.json"));
    if token_file_path.exists() && !force {
        return Err(NewTokenFileError::TokenFileExists {
            path: token_file_path,
        });
    }

    let source_path = format!("{}/{name}.json", config.tokens_dir.display());
    register_in_config(&config_path, &name, &prefix, &source_path, force)?;

    let token_file = TokenFile {
        title: format!("{} Tokens", capitalize(&name)),
        description: Some(format!("Design tokens for {name}")),
        items,
    };

    let json = serde_json::to_string_pretty(&token_file).map_err(NewTokenFileError::Serialize)?;
    fs::write(&token_file_path, json + "\n").map_err(NewTokenFileError::WriteTokenFile)?;

    println!(
        " ✔︎ Created {} at {}",
        format!("{name}.json").green(),
        token_file_path.display()
    );

    Ok(())
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}
