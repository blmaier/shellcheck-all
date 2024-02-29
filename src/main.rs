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
    /// Path to Shellcheck binary
    #[arg(short='s', long, default_value = "shellcheck")]
    shellcheck: PathBuf,

    /// List of arguments for Shellcheck, whitespace seperated
    #[arg(short='a', long, require_equals=true, value_delimiter=' ')]
    shellcheck_args: Option<Vec<String>>,

    #[arg(long, short, default_value="-")]
    output: clio::Output,

    /// Files or directories to check for shell files
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


#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let args = Args::parse();
    let num_threads = num_cpus::get() + 1;

    // Check we have a valid Shellcheck
    let mut shellcheck = Shellcheck::new(args.shellcheck.into_os_string());
    shellcheck.get_version().await?;

    // Build Shellcheck arguments
    if let Some(ref sargs) = args.shellcheck_args {
        shellcheck.add_args(sargs.iter().map(|s| s.to_string().into()));
    }

    // Find shell scripts to check
    let files: Result<Vec<ignore::DirEntry>, ignore::Error> = WalkShellScript::from_iter(args.files).into_iter().collect();
    let files = files?;
    let files_per_process = (files.len() / (num_threads * 16)) + 1;

    // Split list of files into seperate Shellcheck commands
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

    // Run Shellcheck commands and collect output
    let mut comments = Vec::new();
    let mut pool = pool_builder.build(num_threads);
    while let Some(output) = pool.next().await {
        comments.push(output?.stdout);
    }

    let json = merge_shellcheck_json1(comments);
    serde_json::to_writer(args.output, &json)?;

    Ok(())
}
