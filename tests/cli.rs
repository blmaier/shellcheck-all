use assert_cmd::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(PartialEq, Eq, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
enum ShellcheckJson1InsertionPoint {
    AfterEnd,
    BeforeStart,
}

#[derive(PartialEq, Eq, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct ShellcheckJson1Replacement {
    line: u32,
    end_line: u32,
    column: u32,
    end_column: u32,
    insertion_point: ShellcheckJson1InsertionPoint,
    precedence: u32,
    replacement: String,
}

#[derive(PartialEq, Eq, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ShellcheckJson1Fix {
    replacements: Vec<ShellcheckJson1Replacement>,
}

#[derive(PartialEq, Eq, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "lowercase")]
enum ShellcheckJson1Level {
    Info,
    Warning,
    Error,
    Style,
}

#[derive(PartialEq, Eq, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct ShellcheckJson1Comment {
    file: String,
    line: u32,
    end_line: u32,
    column: u32,
    end_column: u32,
    level: ShellcheckJson1Level,
    code: u32,
    message: String,
    fix: ShellcheckJson1Fix,
}

#[derive(PartialEq, Eq, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ShellcheckJson1 {
    comments: Vec<ShellcheckJson1Comment>,
}

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
