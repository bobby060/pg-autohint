use std::fmt;

pub struct PgHintList(Vec<PgHint>);

impl PgHintList {
    ////Concat hint list with sql string
    //// Args:
    //// - sql: sql string to concat with self
    //// Returns:
    //// - sql string with hint list
    pub fn with_sql(self, sql: &str) -> String {
        format!("{} {}", self.to_string(), sql)
    }

    pub fn add_hint(mut self, hint: PgHint) -> Self {
        self.0.push(hint);
        self
    }

    pub fn add_hint_list(mut self, hint_list: PgHintList) -> Self {
        self.0.extend(hint_list.0);
        self
    }
}

impl fmt::Display for PgHintList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "/*+ {} */",
            self.0
                .iter()
                .map(|h| h.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}
pub enum PgHint {
    SeqScan {
        table: String,
    },
    IndexScan {
        table: String,
        index: Option<String>,
    },
    NoSeqScan {
        table: String,
    },
    NoIndexScan {
        table: String,
    },
    NoIndexOnlyScan {
        table: String,
    },
    // Can add more hints here, just be sure to implement fmt::Display for them
}

impl fmt::Display for PgHint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PgHint::SeqScan { table } => f.write_str(&format!("SeqScan({})", table)),
            PgHint::IndexScan { table, index } => {
                if let Some(index) = index {
                    f.write_str(&format!("IndexScan({},{})", table, index))
                } else {
                    f.write_str(&format!("IndexScan({})", table))
                }
            }
            PgHint::NoSeqScan { table } => f.write_str(&format!("NoSeqScan({})", table)),
            PgHint::NoIndexScan { table } => f.write_str(&format!("NoIndexScan({})", table)),
            PgHint::NoIndexOnlyScan { table } => {
                f.write_str(&format!("NoIndexOnlyScan({})", table))
            }
        }
    }
}

// TODO: Add tests
