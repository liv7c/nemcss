//! This module contains the tokens type and implementation.
//! Each token file has a certain structure (title, description, items).
//! This module also contains the types of tokens that are supported (Simple, List for now).

use std::fmt::Display;

use serde::{Deserialize, Serialize};

/// Represents a token file.
/// A token file is a JSON file that contains a title, description and a list of design tokens (items).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TokenFile {
    /// The title of the token file.
    pub title: String,
    /// The description of the design tokens contained in the token file.
    pub description: Option<String>,
    /// The list of design tokens contained in the token file.
    pub items: Vec<TokenItem>,
}

impl TokenFile {
    /// Converts the TokenFile to a HashMap of TokenValue
    pub fn into_tokens(self) -> Vec<(String, TokenValue)> {
        self.items
            .into_iter()
            .map(|item| (item.name, item.value))
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
#[derive(Debug, Clone, PartialEq)]
pub enum TokenValue {
    /// Represents a simple value.
    Simple(String),
    /// Represents a list of values.
    List(Vec<String>),
}

/// Implements the Serialize trait for TokenValue.
/// This allows serializing a TokenValue::Simple into a JSON string and a TokenValue::List into an array.
impl Serialize for TokenValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            TokenValue::Simple(val) => serializer.serialize_str(val),
            TokenValue::List(items) => items.serialize(serializer),
        }
    }
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
            Value::Array(a) => {
                let items = a
                    .into_iter()
                    .map(|v| match v {
                        Value::String(s) => s,
                        other => other.to_string(),
                    })
                    .collect();
                Ok(TokenValue::List(items))
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_token_value_serializes_as_plain_string() {
        let value = TokenValue::Simple("8px".to_string());
        assert_eq!(serde_json::to_string(&value).unwrap(), r#""8px""#);
    }

    #[test]
    fn list_token_value_serializes_as_array() {
        let value = TokenValue::List(vec!["Arial".to_string(), "Helvetica".to_string()]);

        assert_eq!(
            serde_json::to_string(&value).unwrap(),
            r#"["Arial","Helvetica"]"#
        );
    }

    #[test]
    fn token_value_serializes_and_deserializes_correctly() {
        let original = TokenValue::Simple("8px".to_string());
        let json = serde_json::to_string(&original).unwrap();
        let deserialized_val: TokenValue = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized_val, original);
    }
}
