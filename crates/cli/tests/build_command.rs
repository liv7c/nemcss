use assert_fs::TempDir;
use assert_fs::prelude::*;
use predicates::prelude::*;

/// Test environment helper
struct TestCmdHelper {
    temp_dir: TempDir,
    cmd: assert_cmd::Command,
}

impl TestCmdHelper {
    /// Create a new test command helper
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let bin_path = assert_cmd::cargo::cargo_bin!("nemcss");

        let cmd = assert_cmd::Command::new(bin_path);
        Ok(Self { temp_dir, cmd })
    }

    /// Add common design tokens that most tests need
    fn with_standard_design_tokens(self) -> Result<Self, Box<dyn std::error::Error>> {
        self.temp_dir.child("nemcss.config.json").write_str(
            r#"
        {
            "content": ["src/**/*.html"]
        }
        "#,
        )?;

        self.temp_dir.child("design-tokens").create_dir_all()?;

        self.temp_dir
            .child("design-tokens")
            .child("colors.json")
            .write_str(
                r##"{
    "title": "colors",
    "items": [
        {"name": "primary", "value": "#ff0000"},
        {"name": "secondary", "value": "#00ff00"}
    ]
}"##,
            )?;

        self.temp_dir
            .child("design-tokens")
            .child("spacings.json")
            .write_str(
                r##"{
    "title": "spacings",
    "items": [
        {"name": "sm", "value": "0.5rem"},
        {"name": "md", "value": "1rem"}
    ]
}"##,
            )?;

        Ok(self)
    }

    /// Add explicit utilities
    fn with_explicit_utilities_for_colors_spacings(
        self,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        self.temp_dir.child("nemcss.config.json").write_str(
            r#"
        {
            "content": ["src/**/*.html"],
            "theme": {
                "colors": {
                    "source": "design-tokens/colors.json",
                    "utilities": [
                        { "prefix": "text", "property": "color" },
                        { "prefix": "bg",   "property": "background-color" },
                        { "prefix": "bc",   "property": "border-color" }
                    ]
                },
                "spacings": {
                    "source": "design-tokens/spacings.json",
                    "utilities": [
                        { "prefix": "p", "property": "padding" },
                        { "prefix": "m", "property": "margin" }
                    ]
                }
            }
        }
        "#,
        )?;

        Ok(self)
    }

    /// Add viewport tokens for responsive tests
    fn with_standard_viewport_tokens(self) -> Result<Self, Box<dyn std::error::Error>> {
        self.temp_dir
            .child("design-tokens")
            .child("viewports.json")
            .write_str(
                r#"{
    "title": "viewports",
    "items": [
        {"name": "sm", "value": "640px"},
        {"name": "md", "value": "768px"},
        {"name": "lg", "value": "1024px"},
        {"name": "xl", "value": "1280px"}
    ]
}"#,
            )?;
        Ok(self)
    }

    /// Add a design token file to the test directory
    fn with_input_css_file(self, content: &str) -> Result<Self, Box<dyn std::error::Error>> {
        self.temp_dir.child("input.css").write_str(content)?;
        Ok(self)
    }

    /// Add a design token file to the test directory
    fn with_content_file(
        self,
        path: &str,
        content: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        self.temp_dir.child(path).write_str(content)?;
        Ok(self)
    }

    /// Run the build command
    fn run_build_command(&mut self) -> assert_cmd::assert::Assert {
        self.cmd
            .current_dir(&self.temp_dir)
            .arg("build")
            .arg("--input")
            .arg("input.css")
            .arg("--output")
            .arg("output.css")
            .assert()
    }

    /// Read output CSS content
    fn output_css(&self) -> String {
        std::fs::read_to_string(self.temp_dir.child("output.css")).unwrap()
    }
}

#[test]
fn test_build_generates_css_with_only_used_classes() {
    let mut test_setup = TestCmdHelper::new()
        .unwrap()
        .with_standard_design_tokens()
        .unwrap()
        .with_explicit_utilities_for_colors_spacings()
        .unwrap()
        .with_content_file(
            "src/index.html",
            r#"
            <div class="text-primary">Primary</div>
            <div class="text-secondary">Secondary</div>
            <div class="bg-secondary">Primary</div>
            <div class="p-sm">Margin</div>
            "#,
        )
        .unwrap()
        .with_input_css_file(
            r#"
        @nemcss base;

        .custom-class {
            color: red;
        }

        "#,
        )
        .unwrap();

    test_setup.run_build_command().success();

    let css_content = test_setup.output_css();

    // Assert used classes are present
    assert!(
        css_content.contains(".text-primary"),
        "Missing .text-primary"
    );
    assert!(
        css_content.contains(".text-secondary"),
        "Missing .text-secondary"
    );
    assert!(
        css_content.contains(".bg-secondary"),
        "Missing .bg-secondary"
    );
    assert!(css_content.contains(".p-sm"), "Missing .p-sm");
    assert!(
        css_content.contains(".custom-class"),
        "Missing .custom-class from input CSS"
    );

    // Assert unused classes are NOT present
    assert!(
        !css_content.contains(".bg-primary"),
        "Should not generate unused .bg-primary"
    );
    assert!(
        !css_content.contains(".m-sm"),
        "Should not generate unused .m-sm"
    );
    assert!(
        !css_content.contains(".m-md"),
        "Should not generate unused .m-md"
    );
    assert!(
        !css_content.contains(".p-md"),
        "Should not generate unused .p-md"
    );
}

#[test]
fn test_build_generate_only_used_responsive_utilities() {
    let mut test_setup = TestCmdHelper::new()
        .unwrap()
        .with_standard_design_tokens()
        .unwrap()
        .with_explicit_utilities_for_colors_spacings()
        .unwrap()
        .with_content_file(
            "src/index.html",
            r#"
        <div class="text-primary md:text-secondary">Primary</div>
        <div class="bg-primary lg:bg-secondary">Primary</div>
            "#,
        )
        .unwrap()
        .with_input_css_file(
            r#"
            @nemcss base;
            "#,
        )
        .unwrap()
        .with_standard_viewport_tokens()
        .unwrap();

    // Run build command
    test_setup.run_build_command().success();

    let output_file = test_setup.temp_dir.child("output.css");
    output_file.assert(predicate::path::is_file());

    let css_content = test_setup.output_css();

    assert!(
        css_content.contains("@media (min-width: 768px)"),
        "Missing media query for md viewport"
    );
    assert!(
        css_content.contains(".md\\:text-secondary"),
        "Missing .md:text-secondary"
    );
    assert!(
        css_content.contains("@media (min-width: 1024px)"),
        "Missing media query for lg viewport"
    );
    assert!(
        css_content.contains(".lg\\:bg-secondary"),
        "Missing .lg:bg-neutral-200"
    );

    // check unused responsive classes are not generated
    assert!(
        !css_content.contains("@media (min-width: 640px)"),
        "Should not generate media query for unused sm viewport"
    );

    assert!(
        !css_content.contains("@media (min-width: 1280px)"),
        "Should not generate media query for unused xl viewport"
    );
}
