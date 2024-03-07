use std::ffi::OsString;
use std::process::Stdio;
use tokio::process::Command;
use which::which;
use std::path::PathBuf;
use std::ffi::OsStr;

#[derive(Clone, Debug, strum::Display, strum::EnumString, clap::ValueEnum)]
#[strum(serialize_all = "lowercase")]
pub enum ShellcheckFormat {
    Checkstyle,
    Diff,
    GCC,
    JSON,
    JSON1,
    Quiet,
    TTY,
}

#[derive(Clone, Debug)]
pub struct Shellcheck {
    program: PathBuf,
    args: Vec<OsString>,
    format: ShellcheckFormat,
}

impl Shellcheck {
    pub fn new<T: AsRef<OsStr>>(binary_name: T, format: ShellcheckFormat) -> anyhow::Result<Self> {
        let program = which(binary_name)?;
        Ok(Self {
            program,
            args: Vec::new(),
            format,
        })
    }

    pub fn add_args<T>(&mut self, args: T) -> &Self
    where
        T: IntoIterator<Item = OsString>,
    {
        self.args.extend(args);
        self
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
