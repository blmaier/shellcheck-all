pub mod format {
    use serde::{Deserialize, Serialize};
    use std::collections::BTreeSet;

    #[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Deserialize, Serialize)]
    #[serde(deny_unknown_fields, rename_all = "camelCase")]
    enum ShellcheckJson1InsertionPoint {
        AfterEnd,
        BeforeStart,
    }

    #[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Deserialize, Serialize)]
    #[serde(deny_unknown_fields, rename_all = "camelCase")]
    struct ShellcheckJson1Replacement {
        line: u32,
        end_line: u32,
        column: u32,
        end_column: u32,
        insertion_point: ShellcheckJson1InsertionPoint,
        precedence: u32,
        replacement: String,
    }

    #[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Deserialize, Serialize)]
    #[serde(deny_unknown_fields)]
    struct ShellcheckJson1Fix {
        replacements: Vec<ShellcheckJson1Replacement>,
    }

    #[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Deserialize, Serialize)]
    #[serde(deny_unknown_fields, rename_all = "lowercase")]
    enum ShellcheckJson1Level {
        Error,
        Warning,
        Info,
        Style,
    }

    #[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Deserialize, Serialize)]
    #[serde(deny_unknown_fields, rename_all = "camelCase")]
    struct ShellcheckJson1Comment {
        file: String,
        line: u32,
        column: u32,
        code: u32,
        level: ShellcheckJson1Level,
        message: String,
        end_line: u32,
        end_column: u32,
        fix: Option<ShellcheckJson1Fix>,
    }

    #[derive(PartialEq, Eq, Debug, Default, Deserialize, Serialize)]
    #[serde(deny_unknown_fields)]
    pub struct ShellcheckJson1 {
        comments: BTreeSet<ShellcheckJson1Comment>,
    }

    impl ShellcheckJson1 {
        pub fn new() -> Self {
            Self {
                comments: Vec::new(),
            }
        }

        pub fn push(&mut self, value: Self) {
            self.comments.extend(value.comments);
        }
    }
}
