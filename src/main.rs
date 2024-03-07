use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use shellcheck_all::format::ShellcheckJson1;

mod command_pool;
use crate::command_pool::CommandPool;

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

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let args = Args::parse();
    let num_threads = num_cpus::get() + 1;

    // Check we have a valid Shellcheck
    let mut shellcheck = Shellcheck::new(args.shellcheck)?;
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
    let mut pool = CommandPool::new(num_threads);
    for files_chunk in files.chunks(files_per_process) {
        let files: Vec<std::ffi::OsString> = files_chunk.iter().map(|x| x.path().into()).collect();
        let command = shellcheck.check_files(files.clone());
        pool.spawn(command, files);
    }

    // Run Shellcheck commands and collect output
    let mut comments = ShellcheckJson1::default();
    while let Some((files, output)) = pool.next().await {
        //println!("files: {:#?}", files);
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
            comments.push(serde_json::from_slice(&output.stdout)?);
        }
    }

    serde_json::to_writer(args.output, &comments)?;

    Ok(())
}
