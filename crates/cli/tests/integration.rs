use assert_fs::TempDir;
use assert_fs::prelude::*;
use config::CONFIG_FILE_NAME;
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
fn test_help() {
    let (mut cmd, _) = setup_cmd().unwrap();

    cmd.arg("-h")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage: nemcss <COMMAND>"))
        .stderr("");
}

#[test]
fn test_version() {
    let (mut cmd, _) = setup_cmd().unwrap();

    cmd.arg("-V")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")))
        .stderr("");
}

#[test]
fn test_init_generates_config_file_with_correct_content() {
    let (mut cmd, temp_dir) = setup_cmd().unwrap();

    cmd.current_dir(&temp_dir)
        .arg("init")
        .assert()
        .success()
        .stderr("");

    let expected_config_file_content = include_str!("../src/templates/nemcss.config.json");

    temp_dir
        .child("nemcss.config.json")
        .assert(predicate::path::is_file())
        .assert(predicate::str::contains(expected_config_file_content));
}

#[test]
fn test_init_generates_tokens_dir_with_correct_content() {
    let (mut cmd, temp_dir) = setup_cmd().unwrap();

    cmd.current_dir(&temp_dir).arg("init").assert().success();

    temp_dir
        .child("design-tokens")
        .assert(predicate::path::is_dir());

    let expected_spacings_file_content = include_str!("../src/templates/spacings.json");
    temp_dir
        .child("design-tokens")
        .child("spacings.json")
        .assert(predicate::path::is_file())
        .assert(predicate::str::contains(expected_spacings_file_content));

    let expected_colors_file_content = include_str!("../src/templates/colors.json");
    temp_dir
        .child("design-tokens")
        .child("colors.json")
        .assert(predicate::path::is_file())
        .assert(predicate::str::contains(expected_colors_file_content));
}

#[test]
fn test_init_shows_error_if_config_file_already_exists() {
    let (mut cmd, temp_dir) = setup_cmd().unwrap();

    cmd.current_dir(&temp_dir)
        .arg("init")
        .assert()
        .success()
        .stderr("");

    let (mut cmd, _) = setup_cmd().unwrap();
    cmd.current_dir(&temp_dir)
        .arg("init")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "the configuration file already exists",
        ));
}

#[test]
fn test_init_skips_existing_design_tokens_dir() {
    let (mut cmd, temp_dir) = setup_cmd().unwrap();

    temp_dir.child("design-tokens").create_dir_all().unwrap();

    cmd.current_dir(&temp_dir)
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains("already exists, skipping"));

    // Should not create the example tokens files (colors.json and spacings.json)
    temp_dir
        .child("design-tokens")
        .child("spacings.json")
        .assert(predicate::path::missing());

    temp_dir
        .child("design-tokens")
        .child("colors.json")
        .assert(predicate::path::missing());
}

#[test]
fn test_new_token_file_appears_in_help() {
    let (mut cmd, _) = setup_cmd().unwrap();

    cmd.arg("-h")
        .assert()
        .success()
        .stdout(predicate::str::contains("new-token-file"));
}

#[test]
fn test_new_token_file_rejects_values_combined_with_step() {
    let (mut cmd, temp_dir) = setup_cmd().unwrap();

    cmd.current_dir(&temp_dir)
        .args([
            "new-token-file",
            "spacing",
            "--values",
            "8,16",
            "--step",
            "8",
            "--count",
            "4",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}

#[test]
fn test_new_token_file_rejects_step_without_count() {
    let (mut cmd, temp_dir) = setup_cmd().unwrap();

    cmd.current_dir(&temp_dir)
        .args(["new-token-file", "spacing", "--step", "8"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_new_token_file_rejects_start_without_step() {
    let (mut cmd, temp_dir) = setup_cmd().unwrap();

    cmd.current_dir(&temp_dir)
        .args([
            "new-token-file",
            "spacing",
            "--start",
            "0.5",
            "--count",
            "3",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_new_token_file_errors_without_config_file() {
    let (mut cmd, temp_dir) = setup_cmd().unwrap();

    cmd.current_dir(&temp_dir)
        .args([
            "new-token-file",
            "radius",
            "--unit",
            "px",
            "--values",
            "2, 4, 8",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("nemcss init"));
}

#[test]
fn test_new_token_file_creates_token_file() {
    let (mut cmd, temp_dir) = setup_cmd().unwrap();

    cmd.current_dir(&temp_dir).arg("init").assert().success();

    let (mut cmd, _) = setup_cmd().unwrap();

    cmd.current_dir(&temp_dir)
        .args([
            "new-token-file",
            "radius",
            "--unit",
            "px",
            "--values",
            "2, 4, 8",
            "--names",
            "xs,sm,md",
        ])
        .assert()
        .success();

    let expected = r#"{
  "title": "Radius Tokens",
  "description": "Design tokens for radius",
  "items": [
    {
      "name": "xs",
      "value": "2px"
    },
    {
      "name": "sm",
      "value": "4px"
    },
    {
      "name": "md",
      "value": "8px"
    }
  ]
}"#;
    temp_dir
        .child("design-tokens")
        .child("radius.json")
        .assert(predicate::path::is_file())
        .assert(predicate::str::contains(expected));
}

#[test]
fn test_new_token_file_accepts_css_function_values() {
    let (mut cmd, temp_dir) = setup_cmd().unwrap();

    cmd.current_dir(&temp_dir).arg("init").assert().success();

    let (mut cmd, _) = setup_cmd().unwrap();

    cmd.current_dir(&temp_dir)
        .args([
            "new-token-file",
            "font-size",
            "--unit",
            "rem",
            "--values",
            "1,clamp(1.5rem, 1rem + 2vw, 2.5rem)",
            "--names",
            "md, fluid",
        ])
        .assert()
        .success();

    temp_dir
        .child("design-tokens")
        .child("font-size.json")
        .assert(predicate::str::contains(r#"value": "1rem""#))
        .assert(predicate::str::contains(
            r#"value": "clamp(1.5rem, 1rem + 2vw, 2.5rem)""#,
        ));
}

#[test]
fn test_new_token_file_refuses_to_overwrite_without_force_flag() {
    let (mut cmd, temp_dir) = setup_cmd().unwrap();

    cmd.current_dir(&temp_dir).arg("init").assert().success();

    let (mut cmd, _) = setup_cmd().unwrap();

    cmd.current_dir(&temp_dir)
        .args([
            "new-token-file",
            "spacings",
            "--unit",
            "px",
            "--values",
            "8,16",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--force"));
}

#[test]
fn test_new_token_file_overwrites_with_force() {
    let (mut cmd, temp_dir) = setup_cmd().unwrap();

    cmd.current_dir(&temp_dir).arg("init").assert().success();

    let (mut cmd, _) = setup_cmd().unwrap();

    cmd.current_dir(&temp_dir)
        .args([
            "new-token-file",
            "spacings",
            "--unit",
            "px",
            "--values",
            "33",
            "--force",
        ])
        .assert()
        .success();

    temp_dir
        .child("design-tokens")
        .child("spacings.json")
        .assert(predicate::str::contains(r#""value": "33px""#));
}

#[test]
fn test_new_token_file_registers_theme_entry() {
    let (mut cmd, temp_dir) = setup_cmd().unwrap();
    cmd.current_dir(&temp_dir).arg("init").assert().success();

    let (mut cmd, _) = setup_cmd().unwrap();

    cmd.current_dir(&temp_dir)
        .args([
            "new-token-file",
            "radius",
            "--unit",
            "px",
            "--values",
            "2,4,8",
        ])
        .assert()
        .success();

    let config_content = std::fs::read_to_string(temp_dir.child(CONFIG_FILE_NAME).path()).unwrap();
    let config: serde_json::Value = serde_json::from_str(&config_content).unwrap();

    assert_eq!(config["theme"]["radius"]["prefix"], "radius");
    assert_eq!(
        config["theme"]["radius"]["source"],
        "design-tokens/radius.json"
    );
}

#[test]
fn test_new_token_file_respects_custom_prefix() {
    let (mut cmd, temp_dir) = setup_cmd().unwrap();
    cmd.current_dir(&temp_dir).arg("init").assert().success();

    let (mut cmd, _) = setup_cmd().unwrap();

    cmd.current_dir(&temp_dir)
        .args([
            "new-token-file",
            "font-size",
            "--unit",
            "rem",
            "--values",
            "1,1.25",
            "--names",
            "sm, md",
            "--prefix",
            "text",
        ])
        .assert()
        .success();

    let config_content =
        std::fs::read_to_string(temp_dir.child("nemcss.config.json").path()).unwrap();
    let config: serde_json::Value = serde_json::from_str(&config_content).unwrap();
    assert_eq!(config["theme"]["font-size"]["prefix"], "text");
}
