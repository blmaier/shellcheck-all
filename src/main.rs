use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

mod command_pool;
use crate::command_pool::CommandPoolBuilder;

mod shellcheck;
use crate::shellcheck::Shellcheck;

mod walk_scripts;
use crate::walk_scripts::WalkShellScript;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, default_value = "shellcheck")]
    shellcheck: PathBuf,

    #[arg(default_value = "./")]
    files: Vec<PathBuf>,
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


#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let num_threads = num_cpus::get() + 1;

    let shellcheck = Shellcheck::new(args.shellcheck.into_os_string());
    shellcheck.get_version().await?;

    let files: Result<Vec<ignore::DirEntry>, ignore::Error> = WalkShellScript::from_iter(args.files).into_iter().collect();
    let files = files?;
    let files_per_process = (files.len() / (num_threads * 16)) + 1;

    let mut pool_builder = CommandPoolBuilder::new();
    for files_chunk in files.chunks(files_per_process) {
        let command = shellcheck
            .check_files(
                files_chunk.iter().map(
                    |x| x.path().into()
                )
            );
        pool_builder.command(command);
    }

    let mut comments: Vec<serde_json::Value> = Vec::new();

    let mut pool = pool_builder.build(num_threads);
    while let Some(output) = pool.next().await {
        if let Some(next) = extract_comments(&output?.stdout) {
            comments.extend(next);
        }
    }

    let json = serde_json::json!({"comments": comments});
    println!("{}", json);

    Ok(())
}
