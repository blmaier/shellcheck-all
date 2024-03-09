use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

mod command_pool;
use crate::command_pool::CommandPool;

mod shellcheck;
use crate::shellcheck::{Shellcheck, ShellcheckArgs};

mod walk_scripts;
use crate::walk_scripts::WalkShellScript;

#[derive(Parser, Debug)]
struct Args {
    /// Path to Shellcheck binary
    #[arg(long, default_value = "shellcheck")]
    shellcheck: PathBuf,

    #[command(flatten)]
    shellcheck_args: ShellcheckArgs,

    #[arg(long, short, default_value="-")]
    output: clio::Output,

    /// Files or directories to check for shell files
    #[arg(default_value = "./")]
    files: Vec<PathBuf>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let args = Args::parse();

    let num_threads = num_cpus::get() + 1;

    // Check we have a valid Shellcheck
    let shellcheck = Shellcheck::new(args.shellcheck, args.shellcheck_args)?;
    shellcheck.get_version().await?;

    // Find shell scripts to check
    let files: Result<Vec<ignore::DirEntry>, ignore::Error> = WalkShellScript::from_iter(args.files).into_iter().collect();
    let files = files?;
    let files_per_process = (files.len() / (num_threads * 16)) + 1;

    // Split list of files into seperate Shellcheck commands
    let mut pool = CommandPool::new(num_threads);
    for files_chunk in files.chunks(files_per_process) {
        let files: Vec<std::ffi::OsString> = files_chunk.iter().map(|x| x.path().into()).collect();
        let command = shellcheck.check_files(files.clone());
        pool.spawn(command, files);
    }

    // Run Shellcheck commands and collect output
    let mut comments = shellcheck.formatter();
    while let Some((files, output)) = pool.next().await {
        let output = output.expect("Internal command error running Shellcheck");
        if !output.stderr.is_empty() {
            if files.len() > 1 {
                // Other files in this run may be valid
                // Run Shellcheck on each file individually
                for file in files {
                    let filev = vec!(file);
                    let command = shellcheck.check_files(filev.clone());
                    pool.spawn(command, filev);
                }
            } else {
                let stderr = std::str::from_utf8(&output.stderr)?;
                eprintln!("Shellcheck error on {}", files[0].to_str().unwrap());
                eprintln!("{}", stderr);
            }
        } else {
            comments.push_slice(&output.stdout)?;
        }
    }

    comments.to_writer(args.output)?;

    Ok(())
}
