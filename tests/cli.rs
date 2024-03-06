use assert_cmd::prelude::*;
use shellcheck_all::format::ShellcheckJson1;
use std::path::{Path, PathBuf};
use std::process::Command;

fn get_wild_corpus(path: &str) -> PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let wild_corpus = manifest_dir.join("wild-corpus");
    wild_corpus.join(Path::new(path))
}

fn compare_file(path: &str) {
    let shellcheck = which::which("shellcheck").unwrap();
    let file = get_wild_corpus(path);

    let mut shellcheck_cmd = Command::new(&shellcheck);
    shellcheck_cmd
        .arg("--format")
        .arg("json1")
        .arg("--")
        .arg(&file);
    let shellcheck_assert = shellcheck_cmd.assert().failure();
    let shellcheck_output: ShellcheckJson1 =
        serde_json::from_slice(&shellcheck_assert.get_output().stdout).unwrap();

    let mut cmd = Command::cargo_bin("shellcheck-all").unwrap();
    cmd.arg("--shellcheck").arg(&shellcheck).arg("--").arg(file);
    let cmd_assert = cmd.assert().success();
    let cmd_output: ShellcheckJson1 =
        serde_json::from_slice(&cmd_assert.get_output().stdout).unwrap();

    assert_eq!(shellcheck_output, cmd_output);
}

#[test]
fn one_file() {
    compare_file("boringssl/crypto/lhash/make_macros.sh");
}
