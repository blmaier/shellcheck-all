use assert_cmd::prelude::*;
use std::path::{Path, PathBuf};
use std::process::Command;

fn get_wild_corpus(path: &str) -> PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let wild_corpus = manifest_dir.join("wild-corpus");
    wild_corpus.join(Path::new(path))
}

fn compare_file(path: &str, format: &str) {
    let shellcheck = which::which("shellcheck").unwrap();
    let file = get_wild_corpus(path);

    let mut shellcheck_cmd = Command::new(&shellcheck);
    shellcheck_cmd
        .arg("--format")
        .arg(format)
        .arg("--")
        .arg(&file);
    let shellcheck_assert = shellcheck_cmd.assert().failure();
    let shellcheck_output: serde_json::Value =
        serde_json::from_slice(&shellcheck_assert.get_output().stdout).unwrap();

    let mut cmd = Command::cargo_bin("shellcheck-all").unwrap();
    cmd.arg("--shellcheck")
        .arg(&shellcheck)
        .arg("--format")
        .arg(format)
        .arg("--")
        .arg(file);
    let cmd_assert = cmd.assert().success();
    let cmd_output: serde_json::Value =
        serde_json::from_slice(&cmd_assert.get_output().stdout).unwrap();

    assert_eq!(shellcheck_output, cmd_output);
}

#[test]
fn one_file_json1() {
    compare_file("boringssl/crypto/lhash/make_macros.sh", "json1");
}

#[test]
fn one_file_json() {
    compare_file("boringssl/crypto/lhash/make_macros.sh", "json");
}

#[test]
fn find_by_shebang() {
    compare_file("pixelb-scripts/scripts/errno", "json1");
}

#[test]
fn find_by_extension() {
    compare_file("shell/shflags/shflags_issue_28_test.sh", "json1");
}
