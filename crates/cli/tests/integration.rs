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
