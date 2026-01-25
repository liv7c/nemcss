//! This module contains the tokens type and implementation.
//! Each token file has a certain structure (title, description, items).
//! This module also contains the types of tokens that are supported (Simple, List for now).

use std::collections::HashMap;
use std::fmt::Display;

use serde::{Deserialize, Serialize};

/// Represents a token file.
/// A token file is a JSON file that contains a title, description and a list of design tokens (items).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TokenFile {
    /// The title of the token file.
    pub title: String,
    /// The description of the design tokens contained in the token file.
    pub description: String,
    /// The list of design tokens contained in the token file.
    pub items: Vec<TokenItem>,
}

impl TokenFile {
    /// Converts the TokenFile to a HashMap of TokenValue
    pub fn into_tokens(self) -> HashMap<String, TokenValue> {
        self.items
            .into_iter()
            .map(|item| (item.name.clone(), item.value))
            .collect()
    }
}

/// Represents a design token.
/// A design token is a key-value pair where the key is the name of the token and the value is the value of the token.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TokenItem {
    /// The name of the token.
    pub name: String,
    /// The value of the token.
    pub value: TokenValue,
}

/// Represents the value of a design token.
/// The value can be a simple value or a list of values.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum TokenValue {
    /// Represents a simple value.
    Simple(String),
    /// Represents a list of values.
    List(Vec<String>),
}

/// Implements the Deserialize trait for TokenValue.
/// This allows deserializing a TokenValue from a JSON string or an array of strings to distinguish
/// between simple and list values.
impl<'de> Deserialize<'de> for TokenValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde_json::Value;

        let value = Value::deserialize(deserializer)?;

        match value {
            Value::String(s) => Ok(TokenValue::Simple(s)),
            Value::Array(a) => Ok(TokenValue::List(
                a.into_iter().map(|v| v.to_string()).collect(),
            )),
            _ => Err(serde::de::Error::custom("invalid token value")),
        }
    }
}

impl Display for TokenValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenValue::Simple(value) => write!(f, "{}", value),
            TokenValue::List(values) => write!(f, "[{}]", values.join(", ")),
        }
    }
}
