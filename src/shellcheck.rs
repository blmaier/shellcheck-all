use std::ffi::OsString;
use clap::Parser;
use std::process::Stdio;
use tokio::process::Command;
use which::which;
use std::path::PathBuf;
use std::ffi::OsStr;
use shellcheck_all::format::ShellcheckFormatter;
use shellcheck_all::format::ShellcheckJson1;
use shellcheck_all::format::ShellcheckJson;
use shellcheck_all::format::ShellcheckGcc;

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
    args: ShellcheckArgs,
}

#[derive(Parser, Clone, Debug)]
pub struct ShellcheckArgs {
    /// Include warnings from sourced files (Shellcheck)
    #[arg(short='a', long="check-sourced")]
    check_sourced: bool,

    /// Perform dataflow analysis (Shellcheck)
    #[arg(long)]
    extended_analysis: Option<String>,

    /// Output format (Shellcheck)
    #[arg(short='f', long, default_value_t = ShellcheckFormat::Json1)]
    format: ShellcheckFormat,

    /// Don't look for .shellcheckrc files (Shellcheck)
    #[arg(long)]
    norc: bool,

    /// Prefer the specified configuration file over searching for one (Shellcheck)
    #[arg(long)]
    rcfile: Option<String>,

    /// Specify dialect (Shellcheck)
    #[arg(short='s', long)]
    shell: Option<String>,

    /// Allow 'source' outside of FILES (Shellcheck)
    #[arg(short='x', long="external-sources")]
    external_sources: bool,
}

impl Shellcheck {
    pub fn new<T: AsRef<OsStr>>(binary_name: T, args: ShellcheckArgs) -> anyhow::Result<Self> {
        let program = which(binary_name)?;

        match args.format {
            ShellcheckFormat::Json => (),
            ShellcheckFormat::Json1 => (),
            ShellcheckFormat::Gcc => (),
            x => panic!("Shellcheck format '{}' not supported", x),
        };

        Ok(Self {
            program,
            args,
        })
    }

    pub fn formatter(&self) -> ShellcheckFormatter {
        match &self.args.format {
            ShellcheckFormat::Json => ShellcheckFormatter::Json(ShellcheckJson::default()),
            ShellcheckFormat::Json1 => ShellcheckFormatter::Json1(ShellcheckJson1::default()),
            ShellcheckFormat::Gcc => ShellcheckFormatter::Gcc(ShellcheckGcc::default()),
            x => panic!("Shellcheck format '{}' not supported", x),
        }
    }

    pub fn check_files<T>(&self, files: T) -> Command
    where
        T: IntoIterator<Item = OsString>,
    {
        let mut command = self.create_command();
        if self.args.check_sourced {
            command.arg("--check-sourced");
        }
        if let Some(extan) = &self.args.extended_analysis {
            command.arg("--extended-analysis").arg(extan);
        }
        command.arg("--format").arg(self.args.format.to_string());
        if self.args.norc {
            command.arg("--norc");
        }
        if let Some(rcfile) = &self.args.rcfile {
            command.arg("--rcfile").arg(rcfile);
        }
        if let Some(shell) = &self.args.shell {
            command.arg("--shell").arg(shell);
        }
        if self.args.external_sources {
            command.arg("--external-sources");
        }
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
