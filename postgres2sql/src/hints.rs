use std::fmt;

#[derive(Clone)]
pub struct PgHintList(Vec<PgHint>);

impl PgHintList {
    pub fn new() -> Self {
        PgHintList(vec![])
    }
    //// Concat hint list with sql string
    //// Args:
    //// - sql: sql string to concat with self
    //// Returns:
    //// - sql string with hint list
    pub fn with_sql(self, sql: &str) -> String {
        format!("{} {}", self.to_string(), sql)
    }

    //// Add hint to hint list
    //// Args:
    //// - hint: hint to add
    pub fn add_hint(&mut self, hint: PgHint) {
        self.0.push(hint);
    }

    //// Add hint list to hint list
    //// Args:
    //// - hint_list: hint list to add
    pub fn add_hint_list(&mut self, hint_list: PgHintList) {
        self.0.extend(hint_list.0);
    }

    //// Concat hint list with sql string
    //// Returns:
    //// - length of the hint list
    pub fn size(&self) -> usize {
        self.0.len()
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
                .join(" ")
        )
    }
}

#[derive(Clone)]
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
    ValueScan {
        table: String,
    },
    SubqueryScan {
        table: String,
    },
    // Can add more hints here, just be sure to implement fmt::Display for them
    JoinOrder {
        join_order: String, // Maybe make this a vec string or a tree or something
    },
    HashJoin {
        tables: String,
    },
    MergeJoin {
        tables: String,
    },
    IndexOnlyScan {
        table: String,
    },
    // NestLoop {
    //     tables: String
    // },
}

impl fmt::Display for PgHint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PgHint::SeqScan { table } => f.write_str(&format!("SeqScan({})", table)),
            PgHint::IndexScan { table, index } => {
                if let Some(index) = index {
                    f.write_str(&format!("IndexScan({} {})", table, index))
                } else {
                    f.write_str(&format!("IndexScan({})", table))
                }
            }
            PgHint::NoSeqScan { table } => f.write_str(&format!("NoSeqScan({})", table)),
            PgHint::NoIndexScan { table } => f.write_str(&format!("NoIndexScan({})", table)),
            PgHint::NoIndexOnlyScan { table } => {
                f.write_str(&format!("NoIndexOnlyScan({})", table))
            }
            PgHint::JoinOrder { join_order } => f.write_str(&format!("Leading({})", join_order)),
            PgHint::HashJoin { tables } => f.write_str(&format!("HashJoin({})", tables)),
            PgHint::MergeJoin { tables } => f.write_str(&format!("MergeJoin({})", tables)),
            PgHint::IndexOnlyScan { table } => f.write_str(&format!("IndexOnlyScan({})", table)),
            PgHint::ValueScan { table } => f.write_str(&format!("ValueScan({})", table)),
            PgHint::SubqueryScan { table } => f.write_str(&format!("SubqueryScan({})", table)),
        }
    }
}

// TODO: Add tests
