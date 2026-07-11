use std::fs;

use config::{CONFIG_FILE_NAME, NemCssConfig, TokenFile};
use miette::Result;
use owo_colors::OwoColorize;

use crate::commands::new_token_file::{
    error::NewTokenFileError,
    scale::{ScaleSource, build_items},
};

/// Command to generate a new token file
pub fn new_token_file(
    name: &str,
    source: ScaleSource,
    names: Option<Vec<String>>,
    unit: &str,
    prefix: Option<String>,
    force: bool,
) -> Result<(), NewTokenFileError> {
    let current_dir = std::env::current_dir().map_err(NewTokenFileError::RetrieveCurrentDir)?;

    let config_path = current_dir.join(CONFIG_FILE_NAME);
    if !config_path.exists() {
        return Err(NewTokenFileError::ConfigFileNotFound { path: config_path });
    }

    let config = NemCssConfig::from_path(&config_path).map_err(NewTokenFileError::LoadConfig)?;

    let items = build_items(&source, names.as_deref(), unit)?;

    let tokens_dir = current_dir.join(&config.tokens_dir);
    fs::create_dir_all(&tokens_dir).map_err(NewTokenFileError::CreateTokensDir)?;

    let token_file_path = tokens_dir.join(format!("{name}.json"));
    if token_file_path.exists() && !force {
        return Err(NewTokenFileError::TokenFileExists {
            path: token_file_path,
        });
    }

    let token_file = TokenFile {
        title: format!("{} Tokens", capitalize(name)),
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

    // TODO
    let _ = prefix;

    Ok(())
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}
