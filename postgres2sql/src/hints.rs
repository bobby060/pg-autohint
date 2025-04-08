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
    //// - sql string with hint list, if hint list is empty, original sql is returned
    pub fn with_sql(self, sql: &str) -> String {
        let hints = self.to_string();
        if hints.is_empty() {
            sql.to_string()
        } else {
            format!("{}\n{}", hints, sql)
        }
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
        if self.0.is_empty() {
            return write!(f, "");
        }
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
    NestLoop {
        tables: String
    },
    CardCorrection {
        tables: String,
        card: i64
    }
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
            PgHint::NestLoop { tables } => f.write_str(&format!("NestLoop({})", tables)),
            PgHint::IndexOnlyScan { table } => f.write_str(&format!("IndexOnlyScan({})", table)),
            PgHint::ValueScan { table } => f.write_str(&format!("ValueScan({})", table)),
            PgHint::SubqueryScan { table } => f.write_str(&format!("SubqueryScan({})", table)),
            PgHint::CardCorrection { tables, card } => f.write_str(&format!("Rows({} #{})", tables, card)),
        }
    }
}

// TODO: Add tests

#[cfg(test)]
mod test_hints {
    use super::*;

    #[test]
    fn test_hint_display() {
        let hint = PgHint::SeqScan {
            table: "title_basics".to_string(),
        };
        assert_eq!(hint.to_string(), "SeqScan(title_basics)");

        let hint = PgHint::IndexScan {
            table: "title_basics".to_string(),
            index: Some("idx_title_basics_tconst".to_string()),
        };
        assert_eq!(
            hint.to_string(),
            "IndexScan(title_basics idx_title_basics_tconst)"
        );

        let hint = PgHint::NoSeqScan {
            table: "title_basics".to_string(),
        };
        assert_eq!(hint.to_string(), "NoSeqScan(title_basics)");

        let hint = PgHint::NoIndexScan {
            table: "title_basics".to_string(),
        };
        assert_eq!(hint.to_string(), "NoIndexScan(title_basics)");

        let hint = PgHint::NoIndexOnlyScan {
            table: "title_basics".to_string(),
        };
        assert_eq!(hint.to_string(), "NoIndexOnlyScan(title_basics)");

        let hint = PgHint::JoinOrder {
            join_order: "title_basics, title_ratings".to_string(),
        };
        assert_eq!(hint.to_string(), "Leading(title_basics, title_ratings)");

        let hint = PgHint::HashJoin {
            tables: "title_basics, title_ratings".to_string(),
        };
        assert_eq!(hint.to_string(), "HashJoin(title_basics, title_ratings)");

        let hint = PgHint::MergeJoin {
            tables: "title_basics, title_ratings".to_string(),
        };
        assert_eq!(hint.to_string(), "MergeJoin(title_basics, title_ratings)");

        let hint = PgHint::IndexOnlyScan {
            table: "title_basics".to_string(),
        };
        assert_eq!(hint.to_string(), "IndexOnlyScan(title_basics)");

        let hint = PgHint::ValueScan {
            table: "title_basics".to_string(),
        };
        assert_eq!(hint.to_string(), "ValueScan(title_basics)");

        let hint = PgHint::SubqueryScan {
            table: "title_basics".to_string(),
        };
        assert_eq!(hint.to_string(), "SubqueryScan(title_basics)");
    }

    #[test]
    fn test_hint_list_display() {
        let mut hint_list = PgHintList::new();
        hint_list.add_hint(PgHint::SeqScan {
            table: "title_basics".to_string(),
        });
        hint_list.add_hint(PgHint::IndexScan {
            table: "title_basics".to_string(),
            index: Some("idx_title_basics_tconst".to_string()),
        });
        assert_eq!(
            hint_list.to_string(),
            "/*+ SeqScan(title_basics) IndexScan(title_basics idx_title_basics_tconst) */"
        );
    }
}
