pub mod format {
    use serde::{Deserialize, Serialize};
    use std::collections::BTreeSet;
    use strum::EnumString;

    #[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Deserialize, Serialize)]
    #[serde(deny_unknown_fields, rename_all = "camelCase")]
    enum ShellcheckJsonInsertionPoint {
        AfterEnd,
        BeforeStart,
    }

    #[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Deserialize, Serialize)]
    #[serde(deny_unknown_fields, rename_all = "camelCase")]
    struct ShellcheckJsonReplacement {
        line: u32,
        end_line: u32,
        column: u32,
        end_column: u32,
        insertion_point: ShellcheckJsonInsertionPoint,
        precedence: u32,
        replacement: String,
    }

    #[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Deserialize, Serialize)]
    #[serde(deny_unknown_fields)]
    struct ShellcheckJsonFix {
        replacements: Vec<ShellcheckJsonReplacement>,
    }

    #[derive(
        PartialEq, Eq, PartialOrd, Ord, Debug, strum::Display, EnumString, Deserialize, Serialize,
    )]
    #[serde(deny_unknown_fields, rename_all = "lowercase")]
    #[strum(serialize_all = "lowercase")]
    enum ShellcheckJsonLevel {
        Error,
        Warning,
        Info,
        Style,
        Note, // GCC format only, means either Info or Style
    }

    #[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Deserialize, Serialize)]
    #[serde(deny_unknown_fields, rename_all = "camelCase")]
    struct ShellcheckJsonComment {
        file: String,
        line: u32,
        column: u32,
        code: u32,
        level: ShellcheckJsonLevel,
        message: String,
        end_line: Option<u32>,
        end_column: Option<u32>,
        fix: Option<ShellcheckJsonFix>,
    }

    #[derive(PartialEq, Eq, Debug, Default, Deserialize, Serialize)]
    #[serde(deny_unknown_fields)]
    pub struct ShellcheckJson1 {
        comments: BTreeSet<ShellcheckJsonComment>,
    }

    #[derive(PartialEq, Eq, Debug, Default, Deserialize, Serialize)]
    #[serde(deny_unknown_fields)]
    #[serde(transparent)]
    pub struct ShellcheckJson {
        comments: BTreeSet<ShellcheckJsonComment>,
    }

    #[derive(PartialEq, Eq, Debug, Default)]
    pub struct ShellcheckGcc {
        comments: BTreeSet<ShellcheckJsonComment>,
    }

    pub enum ShellcheckFormatter {
        Json(ShellcheckJson),
        Json1(ShellcheckJson1),
        Gcc(ShellcheckGcc),
    }

    impl ShellcheckFormatter {
        pub fn push_slice(&mut self, value: &[u8]) -> anyhow::Result<()> {
            match self {
                Self::Json1(x) => {
                    let y: ShellcheckJson1 = serde_json::from_slice(value)?;
                    x.comments.extend(y.comments);
                }
                Self::Json(x) => {
                    let y: ShellcheckJson = serde_json::from_slice(value)?;
                    x.comments.extend(y.comments);
                }
                Self::Gcc(x) => {
                    for line in std::str::from_utf8(value)?.lines() {
                        let (_, file, line, column, level, message, code) =
                            lazy_regex::regex_captures!(
                                r#"(.+):(\d+):(\d+): (error|warning|note): (.+) \[SC(\d+)\]"#,
                                line
                            )
                            .unwrap();
                        let comment = ShellcheckJsonComment {
                            file: file.to_string(),
                            line: line.parse()?,
                            column: column.parse()?,
                            code: code.parse()?,
                            level: level.parse()?,
                            message: message.to_string(),
                            end_line: None,
                            end_column: None,
                            fix: None,
                        };
                        x.comments.insert(comment);
                    }
                }
            };
            Ok(())
        }

        pub fn to_writer<W>(&self, mut writer: W) -> anyhow::Result<()>
        where
            W: std::io::Write,
        {
            match self {
                Self::Json1(x) => serde_json::to_writer(writer, &x)?,
                Self::Json(x) => serde_json::to_writer(writer, &x)?,
                Self::Gcc(x) => {
                    for comment in x.comments.iter() {
                        writer.write_fmt(format_args!(
                            "{}:{}:{} {}: {} [SC{}]\n",
                            comment.file,
                            comment.line,
                            comment.column,
                            comment.level,
                            comment.message,
                            comment.code,
                        ))?;
                    }
                }
            };
            Ok(())
        }
    }
}
