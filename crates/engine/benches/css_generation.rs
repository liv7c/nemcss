use divan::AllocProfiler;
use engine::VIEWPORT_TOKEN_PREFIX;
use std::collections::{HashMap, HashSet};

use config::{ResolvedSemanticGroup, ResolvedToken, TokenUtilityConfig, TokenValue};

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

/// Helper to create viewports
fn create_viewports() -> ResolvedToken {
    ResolvedToken {
        tokens: vec![
            ("xs".to_string(), TokenValue::Simple("320px".to_string())),
            ("sm".to_string(), TokenValue::Simple("640px".to_string())),
            ("md".to_string(), TokenValue::Simple("768px".to_string())),
            ("lg".to_string(), TokenValue::Simple("1024px".to_string())),
            ("xl".to_string(), TokenValue::Simple("1280px".to_string())),
            ("2xl".to_string(), TokenValue::Simple("1536px".to_string())),
        ],
        utilities: vec![],
        prefix: VIEWPORT_TOKEN_PREFIX.to_string(),
    }
}

/// Benchmark CSS generation with realistic tokens.
///
/// Tests the performance of the CSS generation process:
/// - 5 categories (colors, spacing, fonts, borders, and radii)
/// - 48 total tokens
/// - 19 utility classes
#[divan::bench]
fn realistic_project(bencher: divan::Bencher) {
    let tokens = create_tokens();
    let viewports = create_viewports();

    bencher.bench(|| {
        let css = engine::generate_css(
            divan::black_box(tokens.values()),
            std::iter::empty(),
            divan::black_box(Some(&viewports)),
            None,
        );
        divan::black_box(css.base_to_css());
        divan::black_box(css.utilities_to_css());
    });
}

/// Benchmark CSS generation with minimal tokens.
///
/// Tests the baseline performance of the CSS generation process:
/// - 1 category with 10 tokens and 2 utility classes
/// - 1 viewport with 3 tokens
#[divan::bench]
fn small_dataset(bencher: divan::Bencher) {
    let mut tokens = HashMap::new();
    let (key, value) = create_token_category("colors", "color", 10, 2);
    let (_, viewport_value) = create_token_category("viewports", VIEWPORT_TOKEN_PREFIX, 3, 0);
    tokens.insert(key, value);

    bencher.bench(|| {
        let css = engine::generate_css(
            divan::black_box(tokens.values()),
            std::iter::empty(),
            divan::black_box(Some(&viewport_value)),
            None,
        );
        divan::black_box(css.base_to_css());
        divan::black_box(css.utilities_to_css());
    });
}

/// Benchmark CSS generation with large design system.
///
/// Tests the performance of the CSS generation process:
/// - 10 categories
/// - 200 total tokens
/// - 50 utility classes
/// - 8 viewports for responsive utilities
#[divan::bench]
fn large_design_system(bencher: divan::Bencher) {
    let mut tokens = HashMap::new();
    let (_, viewport_value) = create_token_category("viewports", VIEWPORT_TOKEN_PREFIX, 8, 0);

    for i in 0..10 {
        let (key, value) =
            create_token_category(&format!("category-{i}"), &format!("prefix-{i}"), 20, 5);
        tokens.insert(key, value);
    }

    bencher.bench(|| {
        let css = engine::generate_css(
            divan::black_box(tokens.values()),
            std::iter::empty(),
            divan::black_box(Some(&viewport_value)),
            None,
        );
        divan::black_box(css.base_to_css());
        divan::black_box(css.utilities_to_css());
    });
}

/// Helper to create semantic groups referencing tokens from `create_tokens()`.
///
/// Uses the "colors" group to build a few semantic color aliases.
fn create_semantic_groups() -> Vec<ResolvedSemanticGroup> {
    vec![
        ResolvedSemanticGroup {
            prefix: "text".to_string(),
            property: "color".to_string(),
            tokens: vec![
                ("primary".to_string(), "var(--color-0)".to_string()),
                ("secondary".to_string(), "var(--color-1)".to_string()),
                ("muted".to_string(), "var(--color-2)".to_string()),
            ],
        },
        ResolvedSemanticGroup {
            prefix: "bg".to_string(),
            property: "background-color".to_string(),
            tokens: vec![
                ("surface".to_string(), "var(--color-3)".to_string()),
                ("overlay".to_string(), "var(--color-4)".to_string()),
            ],
        },
    ]
}

/// Helper to derive all possible class names from a token map.
/// Mirrors what the engine does: `{utility_prefix}-{token_name}` for every combination.
fn generate_class_names(tokens: &HashMap<String, ResolvedToken>) -> HashSet<String> {
    tokens
        .values()
        .filter(|r| r.prefix != VIEWPORT_TOKEN_PREFIX)
        .flat_map(|token| {
            token.utilities.iter().flat_map(move |utility| {
                token
                    .tokens
                    .iter()
                    .map(move |(token_name, _)| format!("{}-{}", utility.prefix, token_name))
            })
        })
        .collect()
}

/// Parameterized benchmark testing CSS generation scaling.
///
/// Measures performance across different design system sizes to verify linear scaling characteristics.
/// Each test runs with varying category counts.
///
/// Each category contains:
/// - 15 tokens
/// - 5 utility classes
/// - 5 viewports for responsive utilities
///
/// **Test cases**: 1, 3, 5, 8, 10, 12 categories
#[divan::bench(args = [1, 3, 5, 8, 10, 12])]
fn by_category_count(bencher: divan::Bencher, num_categories: usize) {
    let mut tokens = HashMap::new();
    let (_, viewport_value) = create_token_category("viewports", VIEWPORT_TOKEN_PREFIX, 5, 0);
    for i in 0..num_categories {
        let (key, value) =
            create_token_category(&format!("category-{i}"), &format!("prefix-{i}"), 15, 5);
        tokens.insert(key, value);
    }

    bencher.bench(|| {
        let css = engine::generate_css(
            divan::black_box(tokens.values()),
            std::iter::empty(),
            divan::black_box(Some(&viewport_value)),
            None,
        );
        divan::black_box(css.base_to_css());
        divan::black_box(css.utilities_to_css());
    });
}

#[divan::bench]
fn realistic_project_filtered(bencher: divan::Bencher) {
    let tokens = create_tokens();
    let viewports = create_viewports();
    let all_classes = generate_class_names(&tokens);
    let half_count = all_classes.len() / 2;
    let half: HashSet<String> = all_classes.into_iter().take(half_count).collect();

    bencher.bench(|| {
        let css = engine::generate_css(
            divan::black_box(tokens.values()),
            std::iter::empty(),
            divan::black_box(Some(&viewports)),
            divan::black_box(Some(&half)),
        );
        divan::black_box(css.base_to_css());
        divan::black_box(css.utilities_to_css());
    });
}

#[divan::bench]
fn large_design_system_filtered(bencher: divan::Bencher) {
    let mut tokens = HashMap::new();
    let (_, viewport_value) = create_token_category("viewports", VIEWPORT_TOKEN_PREFIX, 8, 0);

    for i in 0..10 {
        let (key, value) =
            create_token_category(&format!("category-{i}"), &format!("prefix-{i}"), 20, 5);
        tokens.insert(key, value);
    }

    let all_classes = generate_class_names(&tokens);
    let half_count = all_classes.len() / 2;
    let half: HashSet<String> = all_classes.into_iter().take(half_count).collect();

    bencher.bench(|| {
        let css = engine::generate_css(
            divan::black_box(tokens.values()),
            std::iter::empty(),
            divan::black_box(Some(&viewport_value)),
            divan::black_box(Some(&half)),
        );
        divan::black_box(css.base_to_css());
        divan::black_box(css.utilities_to_css());
    });
}

/// Benchmark CSS generation combining primitive tokens with semantic groups.
///
/// Same token set as `realistic_project` plus 2 semantic groups (text, bg)
/// with 5 aliases total. Measures the overhead of semantic custom properties
/// and the extra utility classes they produce.
#[divan::bench]
fn realistic_project_with_semantic(bencher: divan::Bencher) {
    let tokens = create_tokens();
    let viewports = create_viewports();
    let semantic_groups = create_semantic_groups();

    bencher.bench(|| {
        let css = engine::generate_css(
            divan::black_box(tokens.values()),
            divan::black_box(semantic_groups.iter()),
            divan::black_box(Some(&viewports)),
            None,
        );
        divan::black_box(css.base_to_css());
        divan::black_box(css.utilities_to_css());
    });
}

/// Benchmark CSS generation for a large design system with semantic groups.
///
/// Same token set as `large_design_system` (10 categories, 200 tokens) plus
/// 2 semantic groups with 5 aliases. Verifies that semantic overhead stays
/// constant regardless of primitive token count.
#[divan::bench]
fn large_design_system_with_semantic(bencher: divan::Bencher) {
    let mut tokens = HashMap::new();
    let (_, viewport_value) = create_token_category("viewports", VIEWPORT_TOKEN_PREFIX, 8, 0);
    for i in 0..10 {
        let (key, value) =
            create_token_category(&format!("category-{i}"), &format!("prefix-{i}"), 20, 5);
        tokens.insert(key, value);
    }
    let semantic_groups = create_semantic_groups();

    bencher.bench(|| {
        let css = engine::generate_css(
            divan::black_box(tokens.values()),
            divan::black_box(semantic_groups.iter()),
            divan::black_box(Some(&viewport_value)),
            None,
        );
        divan::black_box(css.base_to_css());
        divan::black_box(css.utilities_to_css());
    });
}

#[divan::bench(args = [1, 3, 5, 8, 10, 12])]
fn by_category_count_filtered(bencher: divan::Bencher, num_categories: usize) {
    let mut tokens = HashMap::new();
    let (_, viewport_value) = create_token_category("viewports", VIEWPORT_TOKEN_PREFIX, 5, 0);
    for i in 0..num_categories {
        let (key, value) =
            create_token_category(&format!("category-{i}"), &format!("prefix-{i}"), 15, 5);
        tokens.insert(key, value);
    }

    let all_classes = generate_class_names(&tokens);
    let half_count = all_classes.len() / 2;
    let half: HashSet<String> = all_classes.into_iter().take(half_count).collect();

    bencher.bench(|| {
        let css = engine::generate_css(
            divan::black_box(tokens.values()),
            std::iter::empty(),
            divan::black_box(Some(&viewport_value)),
            divan::black_box(Some(&half)),
        );
        divan::black_box(css.base_to_css());
        divan::black_box(css.utilities_to_css());
    });
}
