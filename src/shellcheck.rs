use std::ffi::OsString;
use clap::Parser;
use std::process::Stdio;
use tokio::process::Command;
use which::which;
use std::path::PathBuf;
use std::ffi::OsStr;

#[derive(Clone, Debug, strum::Display, strum::EnumString, clap::ValueEnum)]
#[strum(serialize_all = "lowercase")]
enum ShellcheckFormat {
    Checkstyle,
    Diff,
    Gcc,
    Json,
    Json1,
    Quiet,
    Tty,
}

#[derive(Clone, Debug)]
pub struct Shellcheck {
    program: PathBuf,
    args: Vec<OsString>,
    format: ShellcheckFormat,
}

#[derive(Parser, Debug)]
pub struct ShellcheckArgs {
    /// Output format (Shellcheck)
    #[arg(short='f', long, default_value_t = ShellcheckFormat::Json1)]
    format: ShellcheckFormat,
}

impl Shellcheck {
    pub fn new<T: AsRef<OsStr>>(binary_name: T, args: ShellcheckArgs) -> anyhow::Result<Self> {
        let program = which(binary_name)?;

        match args.format {
            ShellcheckFormat::Json1 => (),
            x => panic!("Shellcheck format '{}' not supported", x),
        };

        Ok(Self {
            program,
            args: Vec::new(),
            format: args.format,
        })
    }

    pub fn check_files<T>(&self, files: T) -> Command
    where
        T: IntoIterator<Item = OsString>,
    {
        let mut command = self.create_command();
        command.args(self.args.clone());
        command.arg("--format").arg(self.format.to_string());
        command.arg("--").args(files);
        command
    }

    fn create_command(&self) -> Command {
        let mut command = Command::new(self.program.clone());
        command.stdin(Stdio::null());
        command
    }

    pub async fn get_version(&self) -> anyhow::Result<String> {
        let mut command = self.create_command();
        command.arg("--version");
        let output = command.output().await?.stdout;
        for line in String::from_utf8(output)?.lines() {
            if let Some((key, value)) = line.split_once(':') {
                if key == "version" {
                    return Ok(value.trim().into());
                }
            }
        }
        Err(anyhow::anyhow!("Failed to detect version number of Shellcheck"))
    }
}
