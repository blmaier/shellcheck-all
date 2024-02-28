use anyhow::Result;
use clap::Parser;
use ignore::{Walk, WalkBuilder};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, default_value = "shellcheck")]
    shellcheck: PathBuf,

    #[arg(default_value = "./")]
    files: Vec<PathBuf>,
}

trait WalkBuilderExt {
    fn from_iter<P: AsRef<Path>, I: IntoIterator<Item = P>>(iter: I) -> Self;
}

impl WalkBuilderExt for WalkBuilder {
    fn from_iter<P: AsRef<Path>, I: IntoIterator<Item = P>>(iter: I) -> Self {
        let mut iter_mut = iter.into_iter();
        let mut builder = WalkBuilder::new(iter_mut.next().unwrap());

        for path in iter_mut {
            builder.add(path);
        }

        builder
    }
}

struct WalkShellScript {
    builder: WalkBuilder,
}

impl WalkShellScript {
    fn from_iter<P: AsRef<Path>, I: IntoIterator<Item = P>>(iter: I) -> Self {
        let mut iter_mut = iter.into_iter();
        let mut builder = WalkBuilder::new(iter_mut.next().unwrap());

        for path in iter_mut {
            builder.add(path);
        }

        WalkShellScript { builder }
    }
}

impl IntoIterator for WalkShellScript {
    type Item = Result<ignore::DirEntry, ignore::Error>;
    type IntoIter = WalkShellScriptIterator;

    fn into_iter(self) -> Self::IntoIter {
        WalkShellScriptIterator {
            walk: self.builder.build(),
        }
    }
}

struct WalkShellScriptIterator {
    walk: Walk,
}

impl Iterator for WalkShellScriptIterator {
    type Item = Result<ignore::DirEntry, ignore::Error>;
    fn next(&mut self) -> Option<Result<ignore::DirEntry, ignore::Error>> {
        loop {
            match self.walk.next() {
                Some(result) => match result {
                    Ok(entry) => {
                        if entry_is_shellscript(&entry) {
                            break Some(Ok(entry));
                        }
                    }
                    Err(x) => break Some(Err(x)),
                },
                None => break None,
            }
        }
    }
}

fn entry_is_file(entry: &ignore::DirEntry) -> bool {
    match entry.file_type() {
        Some(x) => x.is_file(),
        None => false,
    }
}

fn entry_is_shellscript(entry: &ignore::DirEntry) -> bool {
    if entry_is_file(entry) {
        match file_format::FileFormat::from_file(entry.path()) {
            Ok(fmt) => match fmt {
                file_format::FileFormat::ShellScript => true,
                _ => false,
            },
            Err(err) => panic!("File Format Error: {}", err),
        }
    } else {
        false
    }
}

async fn run_shellcheck(shellcheck: PathBuf, paths: Vec<PathBuf>) -> Vec<u8> {
    let output = Command::new(shellcheck)
        .arg("--format=json1")
        .arg("--")
        .args(paths)
        .stdin(Stdio::null())
        .output()
        .await
        .expect("Failed to run Shellcheck");
    output.stdout
}

fn extract_comments(output: &[u8]) -> Option<Vec<serde_json::Value>> {
    let mut json: serde_json::value::Value = serde_json::from_slice(output).unwrap();
    match json.get_mut("comments") {
        Some(comments) => match comments.take() {
            serde_json::Value::Array(vec) => Some(vec),
            _ => None,
        },
        None => None,
    }
}

async fn shellcheck_version(shellcheck: &Path) -> Result<String>
{
    let output = Command::new(shellcheck).arg("--version").stdin(Stdio::null()).output().await?.stdout;
    Ok(String::from_utf8(output)?)
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let num_threads: usize = num_cpus::get() + 1;

    shellcheck_version(&args.shellcheck).await?;

    let mut tasks = tokio::task::JoinSet::new();
    let mut comments: Vec<serde_json::Value> = Vec::new();

    let files: Result<Vec<ignore::DirEntry>, ignore::Error> = WalkShellScript::from_iter(args.files).into_iter().collect();
    let files = files?;
    let files_per_process = (files.len() / (num_threads * 16)) + 1;

    for files_chunk in files.chunks(files_per_process) {
        while tasks.len() >= num_threads {
            let output: Vec<u8> = tasks.join_next().await.unwrap()?;
            if let Some(next) = extract_comments(&output) {
                comments.extend(next);
            }
        }
        let paths = files_chunk.iter().map(|x| x.path().to_path_buf()).collect();
        tasks.spawn(run_shellcheck(args.shellcheck.clone(), paths));
    }

    while let Some(output) = tasks.join_next().await {
        if let Some(next) = extract_comments(&output?) {
            comments.extend(next);
        }
    }

    let json = serde_json::json!({"comments": comments});
    println!("{}", json);

    Ok(())
}
