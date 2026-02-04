use assert_fs::TempDir;
use assert_fs::prelude::*;
use predicates::prelude::*;

/// Create the command to run the binary.
/// It sets up a temporary directory and returns the path to the binary.
fn setup_cmd() -> Result<(assert_cmd::Command, TempDir), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let bin_path = assert_cmd::cargo::cargo_bin!("nemcss");

    let cmd = assert_cmd::Command::new(bin_path);
    Ok((cmd, temp_dir))
}

#[test]
fn test_build_generates_css_with_only_used_classes() {
    let (mut cmd, temp_dir) = setup_cmd().unwrap();

    // Set up nemcss config with content field
    temp_dir
        .child("nemcss.config.json")
        .write_str(
            r#"
        {
            "content": ["src/**/*.html"]
        }
        "#,
        )
        .unwrap();

    // Set up design tokens
    temp_dir.child("design-tokens").create_dir_all().unwrap();

    temp_dir
        .child("design-tokens")
        .child("colors.json")
        .write_str(
            r##"{
    "title": "colors",
    "items": [
        {"name": "primary", "value": "#ff0000"},
        {"name": "secondary", "value": "#00ff00"},
        {"name": "neutral-100", "value": "#c1c1c1"},
        {"name": "neutral-200", "value": "#c2c2c2"}
    ]
}"##,
        )
        .unwrap();
    temp_dir
        .child("design-tokens")
        .child("spacings.json")
        .write_str(
            r##"{
    "title": "spacings",
    "items": [
        {"name": "xs", "value": "0.25rem"},
        {"name": "sm", "value": "0.5rem"},
        {"name": "md", "value": "1rem"},
        {"name": "lg", "value": "1.5rem"},
        {"name": "xl", "value": "2rem"}
    ]
}"##,
        )
        .unwrap();

    // Create content file with
    temp_dir
        .child("src")
        .child("index.html")
        .write_str(
            r#"

        <div class="text-primary">Primary</div>
        <div class="text-secondary">Secondary</div>
        <div class="bg-neutral-100">Primary</div>
        <div class="m-sm">Margin</div>
    "#,
        )
        .unwrap();

    // Create input CSS with nemcss directive
    temp_dir
        .child("input.css")
        .write_str(
            r#"@nemcss base;

.custom-class {
    color: red;
}
"#,
        )
        .unwrap();

    // Run build command
    cmd.current_dir(&temp_dir)
        .arg("build")
        .arg("--input")
        .arg("input.css")
        .arg("--output")
        .arg("output.css")
        .assert()
        .success();

    // Verify output file contains the used classes
    let output_file = temp_dir.child("output.css");
    output_file.assert(predicate::path::is_file());

    // Read the actual CSS content
    let css_content = std::fs::read_to_string(output_file.path()).unwrap();

    // Assert used classes are present
    assert!(css_content.contains(".text-primary"), "Missing .text-primary");
    assert!(
        css_content.contains(".text-secondary"),
        "Missing .text-secondary"
    );
    assert!(
        css_content.contains(".bg-neutral-100"),
        "Missing .bg-neutral-100"
    );
    assert!(css_content.contains(".m-sm"), "Missing .m-sm");
    assert!(
        css_content.contains(".custom-class"),
        "Missing .custom-class from input CSS"
    );

    // Assert unused classes are NOT present
    assert!(
        !css_content.contains(".text-neutral-200"),
        "Should not generate unused .text-neutral-200"
    );
    assert!(
        !css_content.contains(".bg-primary"),
        "Should not generate unused .bg-primary"
    );
    assert!(
        !css_content.contains(".bg-secondary"),
        "Should not generate unused .bg-secondary"
    );
    assert!(
        !css_content.contains(".m-xs"),
        "Should not generate unused .m-xs"
    );
    assert!(
        !css_content.contains(".m-md"),
        "Should not generate unused .m-md"
    );
    assert!(
        !css_content.contains(".m-lg"),
        "Should not generate unused .m-lg"
    );
    assert!(
        !css_content.contains(".m-xl"),
        "Should not generate unused .m-xl"
    );
}
