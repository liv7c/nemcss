use divan::AllocProfiler;
use std::collections::HashMap;

use config::{ResolvedToken, TokenUtilityConfig, TokenValue};

#[global_allocator]
static ALLOC: AllocProfiler = AllocProfiler::system();

fn main() {
    divan::main();
}

/// Helper to create a single token category
fn create_token_category(
    name: &str,
    prefix: &str,
    num_tokens: usize,
    num_utilities: usize,
) -> (String, ResolvedToken) {
    let tokens: Vec<_> = (0..num_tokens)
        .map(|i| {
            (
                format!("{}-{}", prefix, i),
                TokenValue::Simple(format!("value-{i}")),
            )
        })
        .collect();

    let utilities: Vec<_> = (0..num_utilities)
        .map(|i| TokenUtilityConfig {
            prefix: format!("{}-{}", prefix, i),
            property: "property".to_string(),
        })
        .collect();

    (
        name.to_string(),
        ResolvedToken {
            tokens,
            utilities,
            prefix: prefix.to_string(),
        },
    )
}

/// Helper to create multiple token categories
fn create_tokens() -> HashMap<String, ResolvedToken> {
    let mut resolved_tokens = HashMap::new();

    for (name, prefix, tokens, utilities) in [
        ("colors", "color", 15, 2),
        ("spacing", "spacing", 12, 14),
        ("fonts", "font", 8, 1),
        ("borders", "border", 8, 1),
        ("radii", "radius", 5, 1),
    ] {
        let (key, value) = create_token_category(name, prefix, tokens, utilities);
        resolved_tokens.insert(key, value);
    }

    resolved_tokens
}

#[divan::bench]
fn realistic_project() {
    let tokens = create_tokens();
    let css = engine::generate_css(divan::black_box(tokens.values()));
    divan::black_box(css.to_css());
}

#[divan::bench]
fn small_dataset() {
    let mut tokens = HashMap::new();
    let (key, value) = create_token_category("colors", "color", 10, 2);
    tokens.insert(key, value);

    let css = engine::generate_css(divan::black_box(tokens.values()));
    divan::black_box(css.to_css());
}

#[divan::bench]
fn large_design_system() {
    let mut tokens = HashMap::new();

    for i in 0..10 {
        let (key, value) =
            create_token_category(&format!("category-{i}"), &format!("prefix-{i}"), 20, 5);
        tokens.insert(key, value);
    }

    let css = engine::generate_css(divan::black_box(tokens.values()));
    divan::black_box(css.to_css());
}

/// Paremeterized benchmark
#[divan::bench(args = [1, 3, 5, 8, 10, 12])]
fn by_category_count(num_categories: usize) {
    let mut tokens = HashMap::new();
    for i in 0..num_categories {
        let (key, value) =
            create_token_category(&format!("category-{i}"), &format!("prefix-{i}"), 15, 5);
        tokens.insert(key, value);
    }

    let css = engine::generate_css(divan::black_box(tokens.values()));
    divan::black_box(css.to_css());
}
