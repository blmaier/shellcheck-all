pub mod format {
    use serde::{Deserialize, Serialize};
    use std::collections::BTreeSet;

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

    #[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Deserialize, Serialize)]
    #[serde(deny_unknown_fields, rename_all = "lowercase")]
    enum ShellcheckJsonLevel {
        Error,
        Warning,
        Info,
        Style,
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
        end_line: u32,
        end_column: u32,
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

    pub enum ShellcheckFormatter {
        Json(ShellcheckJson),
        Json1(ShellcheckJson1),
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
            };
            Ok(())
        }

        pub fn to_writer<W>(&self, writer: W) -> anyhow::Result<()>
        where
            W: std::io::Write,
        {
            match self {
                Self::Json1(x) => serde_json::to_writer(writer, &x)?,
                Self::Json(x) => serde_json::to_writer(writer, &x)?,
            };
            Ok(())
        }
    }
}
