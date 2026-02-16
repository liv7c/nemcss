use assert_fs::TempDir;
use assert_fs::prelude::*;
use config::CONFIG_FILE_NAME;
use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};
use std::path::{Path, PathBuf};

use divan::AllocProfiler;

#[global_allocator]
static ALLOC: AllocProfiler = AllocProfiler::system();

fn main() {
    divan::main();
}

/// Helper to create a project structure for benchmarking
struct ProjectGenerator {
    /// The path to the temporary directory
    temp_dir: TempDir,
}

impl ProjectGenerator {
    /// Create a new temporary project directory
    fn new() -> Self {
        Self {
            temp_dir: TempDir::new().expect("Failed to create temp directory"),
        }
    }

    /// Returns the path to the temporary directory
    fn dir(&self) -> &Path {
        self.temp_dir.path()
    }

    fn paths(&self) -> (PathBuf, PathBuf) {
        (
            self.temp_dir.path().join("input.css"),
            self.temp_dir.path().join("output.css"),
        )
    }

    /// Generate configuration file
    fn with_config(&self, custom_config: Option<&str>) -> &Self {
        let default_config = r#"{
            "content": ["src/**/*.html"]
        }"#;

        self.temp_dir
            .child(CONFIG_FILE_NAME)
            .write_str(custom_config.unwrap_or(default_config))
            .expect("Failed to write config file content");
        self
    }

    /// Generate input CSS file
    fn with_input_css(&self) -> &Self {
        self.temp_dir
            .child("input.css")
            .write_str(
                r#"
            @nemcss base;

            .custom {
                color: purple;
            }
            "#,
            )
            .expect("Failed to write input.css file with @nemcss directive");
        self
    }

    /// Generate design token files
    fn with_design_tokens(&self, num_categories: usize, tokens_per_category: usize) -> &Self {
        self.temp_dir
            .child("design-tokens")
            .create_dir_all()
            .expect("Failed to create design-tokens directory");

        self.temp_dir
            .child("design-tokens/viewports.json")
            .write_str(
                r#"
            {
                "title": "viewports",
                "items": [
                    {"name": "xs", "value": "320px"},
                    {"name": "sm", "value": "640px"},
                    {"name": "md", "value": "768px"},
                    {"name": "lg", "value": "1024px"},
                    {"name": "xl", "value": "1280px"},
                    {"name": "xxl", "value": "1540px"}
                ]
            }
            "#,
            )
            .expect("Failed to create viewports.json file with viewports json content");

        let token_types = [
            "colors",
            "spacings",
            "fonts",
            "font-sizes",
            "font-weights",
            "shadows",
            "borders",
            "radii",
        ];

        // Generate up to 8 categories of tokens
        for &token_type in token_types.iter().take(num_categories) {
            let mut items = Vec::new();

            for token_idx in 0..tokens_per_category {
                items.push(format!(
                    r#"{{"name": "token-{}", "value": "value-{}"}}"#,
                    token_idx, token_idx
                ));
            }

            let content = format!(
                r#"
                {{
                    "title": "{}",
                    "items": [
                    {}
                    ]
                
                }}
                "#,
                token_type,
                items.join(",\n     ")
            );

            self.temp_dir
                .child(format!("design-tokens/{}.json", token_type))
                .write_str(&content)
                .expect("Failed to write design token file");
        }

        self
    }

    /// Generate content files with classes
    fn with_content_files(
        &self,
        num_files: usize,
        classes_per_file: usize,
        use_responsive: bool,
    ) -> &Self {
        self.temp_dir
            .child("src")
            .create_dir_all()
            .expect("Failed to create src directory");

        let viewports = if use_responsive {
            vec!["", "sm:", "md:", "lg:", "xl:"]
        } else {
            vec![""]
        };

        let utility_patterns = [
            ("text", "token"),    // colors -> text-token-X
            ("bg", "token"),      // colors -> bg-token-X
            ("p", "token"),       // spacings -> p-token-X
            ("m", "token"),       // spacings -> m-token-X
            ("font", "token"),    // fonts -> font-token-X
            ("shadow", "token"),  // shadows -> shadow-token-X
            ("rounded", "token"), // radii -> rounded-token-X
        ];

        // HTML tags with their nesting preferences
        let container_tags = [
            "div", "section", "article", "header", "nav", "main", "aside", "footer",
        ];
        let inline_tags = ["span", "a", "button", "label"];

        for file_idx in 0..num_files {
            // Seed RNG per file for reproducible but varied classes
            let mut rng = StdRng::seed_from_u64(file_idx as u64);
            let mut html_elements = Vec::new();
            let mut total_classes = 0;

            // Generate nested HTML structure
            while total_classes < classes_per_file {
                // Create a container element with children
                let container_tag = container_tags[rng.random_range(0..container_tags.len())];

                // Container gets 2-5 classes
                let container_class_count = rng
                    .random_range(2..=5)
                    .min(classes_per_file - total_classes);
                let container_classes = self.generate_classes(
                    &mut rng,
                    container_class_count,
                    &viewports,
                    &utility_patterns,
                );
                total_classes += container_class_count;

                let mut children = Vec::new();

                // Add 2-4 child elements
                let num_children = rng
                    .random_range(2..=4)
                    .min((classes_per_file - total_classes) / 2 + 1);

                for _ in 0..num_children {
                    if total_classes >= classes_per_file {
                        break;
                    }

                    let is_inline = rng.random_range(0..10) < 3; // 30% chance of inline element
                    let child_tag = if is_inline {
                        inline_tags[rng.random_range(0..inline_tags.len())]
                    } else {
                        container_tags[rng.random_range(0..container_tags.len())]
                    };

                    // Each child gets 1-3 classes
                    let child_class_count = rng
                        .random_range(1..=3)
                        .min(classes_per_file - total_classes);
                    let child_classes = self.generate_classes(
                        &mut rng,
                        child_class_count,
                        &viewports,
                        &utility_patterns,
                    );
                    total_classes += child_class_count;

                    // Sometimes add a nested grandchild
                    if !is_inline && total_classes < classes_per_file && rng.random_range(0..10) < 4
                    {
                        let grandchild_tag = inline_tags[rng.random_range(0..inline_tags.len())];
                        let grandchild_class_count = rng
                            .random_range(1..=2)
                            .min(classes_per_file - total_classes);
                        let grandchild_classes = self.generate_classes(
                            &mut rng,
                            grandchild_class_count,
                            &viewports,
                            &utility_patterns,
                        );
                        total_classes += grandchild_class_count;

                        children.push(format!(
                            r#"    <{} class="{}">
      <{} class="{}">Text content</{}>
    </{}>"#,
                            child_tag,
                            child_classes.join(" "),
                            grandchild_tag,
                            grandchild_classes.join(" "),
                            grandchild_tag,
                            child_tag
                        ));
                    } else {
                        children.push(format!(
                            r#"    <{} class="{}">Content</{}>"#,
                            child_tag,
                            child_classes.join(" "),
                            child_tag
                        ));
                    }
                }

                html_elements.push(format!(
                    r#"  <{} class="{}">
{}
  </{}>"#,
                    container_tag,
                    container_classes.join(" "),
                    children.join("\n"),
                    container_tag
                ));
            }

            let content = format!(
                r#"<!DOCTYPE html>
<html>
<head><title>Test File {}</title></head>
<body>
{}
</body>
</html>"#,
                file_idx,
                html_elements.join("\n")
            );

            self.temp_dir
                .child(format!("src/file-{}.html", file_idx))
                .write_str(&content)
                .expect("Failed to create html file");
        }

        self
    }

    /// Helper to generate random classes
    fn generate_classes(
        &self,
        rng: &mut StdRng,
        count: usize,
        viewports: &[&str],
        utility_patterns: &[(&str, &str)],
    ) -> Vec<String> {
        let mut classes = Vec::with_capacity(count);
        for _ in 0..count {
            let viewport = viewports[rng.random_range(0..viewports.len())];
            let (prefix, token_base) =
                utility_patterns[rng.random_range(0..utility_patterns.len())];
            let token_num = rng.random_range(0..100);
            classes.push(format!(
                "{}{}-{}-{}",
                viewport, prefix, token_base, token_num
            ));
        }
        classes
    }
}

/// Benchmark build command with a small project
///
/// Project characteristics:
/// - 3 token types (colors, spacings, fonts)
/// - 10 tokens per type
/// - 10 content files
/// - 50 classes per file (500 total classes)
/// - No responsive classes (no md:.., lg:...)
#[divan::bench]
fn small_project(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| {
            let project_gen = ProjectGenerator::new();
            project_gen
                .with_design_tokens(3, 10)
                .with_content_files(10, 50, false)
                .with_config(None)
                .with_input_css();

            let (input, output) = project_gen.paths();
            let dir = project_gen.dir().to_path_buf();
            (project_gen, input, output, dir)
        })
        .bench_values(|(project_gen, input, output, dir)| {
            std::env::set_current_dir(&dir).expect("Failed to set current directory to temp dir");
            cli::commands::build(input, output, true)
                .expect("Build command failed during benchmark");
            drop(project_gen);
        });
}

/// Benchmark build command with a medium project
///
/// Project characteristics:
/// - 5 token types (colors, spacings, fonts, font-sizes, font-weights)
/// - 50 tokens per type
/// - 100 content files (increased for better concurrency testing)
/// - 100 classes per file (10,000 total classes)
/// - Includes responsive variants
#[divan::bench]
fn medium_project(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| {
            let project_gen = ProjectGenerator::new();
            project_gen
                .with_design_tokens(5, 50)
                .with_content_files(100, 100, true)
                .with_config(None)
                .with_input_css();

            let (input, output) = project_gen.paths();
            let dir = project_gen.dir().to_path_buf();
            (project_gen, input, output, dir)
        })
        .bench_values(|(project_gen, input, output, dir)| {
            std::env::set_current_dir(&dir).expect("Failed to set current directory to temp dir");
            cli::commands::build(input, output, true)
                .expect("Build command failed during benchmark");
            drop(project_gen);
        });
}

/// Benchmark build command with a large project
///
/// Project characteristics:
/// - 8 token types (all standard types)
/// - 100 tokens per type
/// - 400 content files (increased for realistic file I/O concurrency)
/// - 150 classes per file (60,000 total classes)
/// - Includes responsive variants
#[divan::bench]
fn large_project(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| {
            let project_gen = ProjectGenerator::new();
            project_gen
                .with_design_tokens(8, 100)
                .with_content_files(400, 150, true)
                .with_config(None)
                .with_input_css();

            let (input, output) = project_gen.paths();
            let dir = project_gen.dir().to_path_buf();
            (project_gen, input, output, dir)
        })
        .bench_values(|(project_gen, input, output, dir)| {
            std::env::set_current_dir(&dir).expect("Failed to set current directory to temp dir");
            cli::commands::build(input, output, true)
                .expect("Build command failed during benchmark");
            drop(project_gen);
        });
}

/// Benchmark build command with an extra large project
///
/// Project characteristics:
/// - 8 token types (all standard types)
/// - 150 tokens per type (more variety)
/// - 1000 content files (stress test for file I/O and concurrency)
/// - 100 classes per file (100,000 total classes)
/// - Includes responsive variants
///
/// This benchmark is useful for:
/// - Testing scalability limits
/// - Identifying performance bottlenecks at scale
/// - Verifying concurrency optimizations
#[divan::bench]
fn extra_large_project(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| {
            let project_gen = ProjectGenerator::new();
            project_gen
                .with_design_tokens(8, 150)
                .with_content_files(1000, 100, true)
                .with_config(None)
                .with_input_css();

            let (input, output) = project_gen.paths();
            let dir = project_gen.dir().to_path_buf();
            (project_gen, input, output, dir)
        })
        .bench_values(|(project_gen, input, output, dir)| {
            std::env::set_current_dir(&dir).expect("Failed to set current directory to temp dir");
            cli::commands::build(input, output, true)
                .expect("Build command failed during benchmark");
            drop(project_gen);
        });
}
