use std::ffi::OsString;
use std::process::Stdio;
use tokio::process::Command;

#[derive(Clone)]
pub enum ShellcheckFormats {
    JSON1,
}

#[derive(Clone)]
pub struct Shellcheck {
    program: OsString,
    format: ShellcheckFormats,
}

impl Shellcheck {
    pub fn new(program: OsString) -> Self {
        Self {
            program,
            format: ShellcheckFormats::JSON1,
        }
    }

    pub fn check_files<T>(&self, files: T) -> Command
    where
        T: IntoIterator<Item = OsString>,
    {
        let mut command = self.create_command();
        match self.format {
            ShellcheckFormats::JSON1 => command.arg("--format=json1"),
        };
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
        Ok(String::from_utf8(output)?)
    }
}
