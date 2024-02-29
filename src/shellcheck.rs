use std::ffi::OsString;
use std::process::Stdio;
use tokio::process::Command;

#[derive(Clone, Debug)]
pub enum ShellcheckFormats {
    JSON1,
}

#[derive(Clone, Debug)]
pub struct Shellcheck {
    program: OsString,
    args: Vec<OsString>,
    format: ShellcheckFormats,
}

impl Shellcheck {
    pub fn new(program: OsString) -> Self {
        Self {
            program,
            args: Vec::new(),
            format: ShellcheckFormats::JSON1,
        }
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
