use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_version() {
    Command::cargo_bin("pixelens")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("pixelens"));
}

#[test]
fn test_help() {
    Command::cargo_bin("pixelens")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Linux-native visual search and OCR utility",
        ));
}

#[test]
fn test_grab_help() {
    Command::cargo_bin("pixelens")
        .unwrap()
        .args(["grab", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Select a region"));
}

#[test]
fn test_copy_help() {
    Command::cargo_bin("pixelens")
        .unwrap()
        .args(["copy", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Select a region"));
}

#[test]
fn test_search_help() {
    Command::cargo_bin("pixelens")
        .unwrap()
        .args(["search", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Select a region"));
}

#[test]
fn test_ai_help() {
    Command::cargo_bin("pixelens")
        .unwrap()
        .args(["ai", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Select a region"));
}

#[test]
fn test_translate_help() {
    Command::cargo_bin("pixelens")
        .unwrap()
        .args(["translate", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Select a region"));
}

#[test]
fn test_image_help() {
    Command::cargo_bin("pixelens")
        .unwrap()
        .args(["image", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Select a region"));
}

#[test]
fn test_daemon_help() {
    Command::cargo_bin("pixelens")
        .unwrap()
        .args(["daemon", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Manage the pixelensd daemon"));
}

#[test]
fn test_daemon_start_help() {
    Command::cargo_bin("pixelens")
        .unwrap()
        .args(["daemon", "start", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Start the daemon process"));
}

#[test]
fn test_daemon_status_when_not_running() {
    Command::cargo_bin("pixelens")
        .unwrap()
        .args(["daemon", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Daemon: stopped"));
}

#[test]
fn test_daemon_stop_when_not_running() {
    Command::cargo_bin("pixelens")
        .unwrap()
        .args(["daemon", "stop"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Daemon was not running"));
}

#[test]
fn test_config_show() {
    Command::cargo_bin("pixelens")
        .unwrap()
        .arg("config")
        .assert()
        .success()
        .stdout(predicate::str::contains("Endpoint:"));
}

#[test]
fn test_version_command() {
    Command::cargo_bin("pixelens")
        .unwrap()
        .arg("version")
        .assert()
        .success()
        .stdout(predicate::str::contains("pixelens"));
}

#[test]
fn test_grab_fails_without_daemon() {
    Command::cargo_bin("pixelens")
        .unwrap()
        .arg("grab")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Daemon not running"));
}

#[test]
fn test_copy_fails_without_daemon() {
    Command::cargo_bin("pixelens")
        .unwrap()
        .arg("copy")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Daemon not running"));
}

#[test]
fn test_search_fails_without_daemon() {
    Command::cargo_bin("pixelens")
        .unwrap()
        .arg("search")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Daemon not running"));
}

#[test]
fn test_ai_fails_without_daemon() {
    Command::cargo_bin("pixelens")
        .unwrap()
        .arg("ai")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Daemon not running"));
}

#[test]
fn test_translate_fails_without_daemon() {
    Command::cargo_bin("pixelens")
        .unwrap()
        .args(["translate", "--to", "Spanish"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Daemon not running"));
}

#[test]
fn test_image_fails_without_daemon() {
    Command::cargo_bin("pixelens")
        .unwrap()
        .arg("image")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Daemon not running"));
}
