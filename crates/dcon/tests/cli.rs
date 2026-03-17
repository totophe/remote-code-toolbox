// Integration tests for the dcon binary.
// These run the compiled binary and assert on stdout/stderr/exit code.

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn cmd() -> Command {
    Command::cargo_bin("dcon").unwrap()
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[test]
fn fails_without_devcontainer() {
    let tmp = TempDir::new().unwrap();
    cmd()
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("no .devcontainer folder found"))
        .stderr(predicate::str::contains("hint:"));
}

#[test]
fn fails_with_no_running_container() {
    // The project has a .devcontainer but no matching Docker container will be
    // found (we use a UUID-based folder name that can't match any real container).
    let tmp = TempDir::new().unwrap();
    let project_dir = tmp.path().join("zzz-nonexistent-project-uuid-12345");
    std::fs::create_dir_all(project_dir.join(".devcontainer")).unwrap();

    cmd()
        .current_dir(&project_dir)
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("no running dev container")
                .or(predicate::str::contains("docker not found"))
                .or(predicate::str::contains("docker ps failed")),
        );
}

#[test]
fn help_flag_succeeds() {
    cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("window"));
}

#[test]
fn window_flag_accepted() {
    let tmp = TempDir::new().unwrap();
    let project_dir = tmp.path().join("zzz-nonexistent-project-uuid-99999");
    std::fs::create_dir_all(project_dir.join(".devcontainer")).unwrap();

    let output = cmd()
        .current_dir(&project_dir)
        .args(["-n", "api"])
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("unexpected argument"),
        "clap should accept -n: {stderr}"
    );
}

#[test]
fn stack_flag_accepted() {
    let tmp = TempDir::new().unwrap();
    let project_dir = tmp.path().join("zzz-nonexistent-project-uuid-stack");
    std::fs::create_dir_all(project_dir.join(".devcontainer")).unwrap();

    let output = cmd()
        .current_dir(&project_dir)
        .args(["--stack", "3"])
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("unexpected argument"), "{stderr}");
    assert!(!stderr.contains("invalid value"), "{stderr}");
}

#[test]
fn side_by_side_flag_accepted() {
    let tmp = TempDir::new().unwrap();
    let project_dir = tmp.path().join("zzz-nonexistent-project-uuid-sbs");
    std::fs::create_dir_all(project_dir.join(".devcontainer")).unwrap();

    let output = cmd()
        .current_dir(&project_dir)
        .args(["--side-by-side", "2"])
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("unexpected argument"), "{stderr}");
    assert!(!stderr.contains("invalid value"), "{stderr}");
}

#[test]
fn stack_and_side_by_side_are_mutually_exclusive() {
    let tmp = TempDir::new().unwrap();
    let project_dir = tmp.path().join("zzz-nonexistent-project-uuid-mutex");
    std::fs::create_dir_all(project_dir.join(".devcontainer")).unwrap();

    cmd()
        .current_dir(&project_dir)
        .args(["--stack", "2", "--side-by-side", "2"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("mutually exclusive"));
}

#[test]
fn stack_rejects_value_below_2() {
    cmd()
        .args(["--stack", "1"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("too few").or(predicate::str::contains("invalid value")));
}

#[test]
fn stack_rejects_value_above_max() {
    cmd()
        .args(["--stack", "11"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("maximum").or(predicate::str::contains("invalid value")));
}
