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

fn merge_shellcheck_json1(outputs: Vec<Vec<u8>>) -> serde_json::Value {
    let mut all_comments = Vec::new();
    for output in outputs {
        let mut json: serde_json::value::Value = serde_json::from_slice(&output).unwrap();
        if let Some(comments) = json.get_mut("comments") {
            if let serde_json::Value::Array(vec) = comments.take() {
                all_comments.extend(vec);
            }
        }
    }
    serde_json::json!({"comments": all_comments})
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

    let mut comments = Vec::new();

    let mut pool = pool_builder.build(num_threads);
    while let Some(output) = pool.next().await {
        comments.push(output?.stdout);
    }

    let json = merge_shellcheck_json1(comments);

    println!("{}", json);

    Ok(())
}
