use std::fmt;

use crate::{
    optimize::{optimize_access_method, optimize_join_method},
    postgres2plan::{HashJoin, JoinNode, PlanNode, ScanNode},
};

pub struct PgHintList(Vec<PgHint>);

impl PgHintList {
    pub fn new() -> Self {
        PgHintList(vec![])
    }
    ////Concat hint list with sql string
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
    JoinOrder {
        join_order: String, // Maybe make this a vec string or a tree or something
    },
    HashJoin {
        tables: String,
    },
    MergeJoin {
        tables: String,
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
        }
    }
}

impl PgHintList {
    pub fn parse_hints(&mut self, root: PlanNode) {
        let join_order = self.build_join_hints(root);
        self.add_hint(PgHint::JoinOrder { join_order });
    }

    /// function for building the join order string for Leading hint
    /// by traversing the tree and recording the join order
    /// also record join method of each join as a hint
    fn build_join_hints(&mut self, root: PlanNode) -> String {
        match root {
            PlanNode::HashJoin(hash_join) => self.add_join(JoinNode::HashJoin(hash_join)),
            PlanNode::MergeJoin(merge_join) => self.add_join(JoinNode::MergeJoin(merge_join)),
            // PlanNode::NestedLoopJoin(nested_loop_join) => {
            //     Some(add_join(JoinNode::NestedLoopJoin(nested_loop_join)))
            // }
            PlanNode::SeqScan(seq_scan) => self.add_scan(ScanNode::SeqScan(seq_scan)),
            PlanNode::IndexScan(index_scan) => self.add_scan(ScanNode::IndexScan(index_scan)),
            _ => {
                // if not join or scan nodes, visit children
                let children = root.get_children();
                match children {
                    Some(children) => match children.len() {
                        1 => {
                            return self.build_join_hints(children.get(0).unwrap().clone());
                        }
                        _ => {
                            todo!("handle subqueries where a node like Filter could have 2 child, one for input one for subquery as predicate, 
                                  also need to handle SetOps nodes that can have two children, that's technically also subqueries")
                        }
                    },
                    None => {
                        panic!("Non ScanNode has 0 children")
                    }
                }
            }
        }
    }

    /// add a join order as well as the join algo to the PgHintList
    fn add_join(&mut self, join_node: JoinNode) -> String {
        let left = self.build_join_hints(join_node.get_left().unwrap());
        let right = self.build_join_hints(join_node.get_right().unwrap());
        let join_order = format!("({} {})", left, right);
        let join_method = match join_node {
            JoinNode::HashJoin(_) => PgHint::HashJoin {
                tables: join_order.clone(),
            },
            JoinNode::MergeJoin(_) => PgHint::MergeJoin {
                tables: join_order.clone(),
            }, // // if we want to convert NLJ to Hash blindly
               // JoinNode::NestedLoopJoin(_) => {
               //    PgHint::HashJoin { tables: join_order.clone() };
               // }
        };
        // TODO: now optimize_join_method is a no-op
        self.add_hint(optimize_join_method(join_method));
        join_order
    }

    /// helper function for constructing a join from its left and right scan child
    fn add_scan(&mut self, scan_node: ScanNode) -> String {
        let rel_name = scan_node.get_relation_name();
        let access_method = match scan_node {
            ScanNode::SeqScan(_) => PgHint::SeqScan {
                table: rel_name.clone(),
            },
            ScanNode::IndexScan(_) => {
                let index_name = scan_node.get_index_name().unwrap();
                PgHint::IndexScan {
                    table: rel_name.clone(),
                    index: Some(index_name),
                }
            }
        };
        // TODO: now optimize_access_method is a no-op
        self.add_hint(optimize_access_method(access_method));
        rel_name
    }
}

// TODO: Add tests

/// test for parsing the join order, join method, and access methods
/// and calling into the crate::optimize::optimize_join_method
/// and crate::optimize::optimize_access_method functions
/// finally converting the hints into string
#[cfg(test)]
mod test_parse_hints {
    use crate::postgres2plan::postgres2plan;

    use super::*;
    #[test]
    fn test_threeway_join_q3() {
        let input_path = "resources/test_json/q3.json";
        let mut pg_hint_list = PgHintList::new();
        let input = std::fs::read_to_string(input_path).expect("Failed to read input file");
        let plan_node = postgres2plan(&input).unwrap();

        println!("{:#?}", plan_node);
        pg_hint_list.parse_hints(plan_node);
        let hints = pg_hint_list.with_sql("");
        println!("---\nparsed hints: \n{}", hints);
    }

    #[test]
    fn test_threeway_join_q4() {
        let input_path = "resources/test_json/q4.json";
        let mut pg_hint_list = PgHintList::new();
        let input = std::fs::read_to_string(input_path).expect("Failed to read input file");
        let plan_node = postgres2plan(&input).unwrap();

        println!("{:#?}", plan_node);
        pg_hint_list.parse_hints(plan_node);
        let hints = pg_hint_list.with_sql("");
        println!("---\nparsed hints: \n{}", hints);
    }

    #[test]
    fn test_no_join_q5() {
        let input_path = "resources/test_json/q5.json";
        let mut pg_hint_list = PgHintList::new();
        let input = std::fs::read_to_string(input_path).expect("Failed to read input file");
        let plan_node = postgres2plan(&input).unwrap();

        println!("{:#?}", plan_node);
        pg_hint_list.parse_hints(plan_node);
        let hints = pg_hint_list.with_sql("");
        println!("---\nparsed hints: \n{}", hints);
    }

    #[test]
    fn test_threeway_merge_join_index_scan_q7() {
        let input_path = "resources/test_json/q7.json";
        let mut pg_hint_list = PgHintList::new();
        let input = std::fs::read_to_string(input_path).expect("Failed to read input file");
        let plan_node = postgres2plan(&input).unwrap();

        println!("{:#?}", plan_node);
        pg_hint_list.parse_hints(plan_node);
        let hints = pg_hint_list.with_sql("");
        println!("---\nparsed hints: \n{}", hints);
    }

    /// TODO: need to support SetOps / Subquery to pass this test
    #[ignore]
    #[test]
    fn todo_test_union_q9() {
        let input_path = "resources/test_json/q9.json";
        let mut pg_hint_list = PgHintList::new();
        let input = std::fs::read_to_string(input_path).expect("Failed to read input file");
        let plan_node = postgres2plan(&input).unwrap();

        println!("{:#?}", plan_node);
        pg_hint_list.parse_hints(plan_node);
        let hints = pg_hint_list.with_sql("");
        println!("---\nparsed hints: \n{}", hints);
    }

    /// TODO: need to support Subquery to pass this test
    #[ignore]
    #[test]
    fn todo_test_join_q10() {
        let input_path = "resources/test_json/q10.json";
        let mut pg_hint_list = PgHintList::new();
        let input = std::fs::read_to_string(input_path).expect("Failed to read input file");
        let plan_node = postgres2plan(&input).unwrap();

        println!("{:#?}", plan_node);
        pg_hint_list.parse_hints(plan_node);
        let hints = pg_hint_list.with_sql("");
        println!("---\nparsed hints: \n{}", hints);
    }
}
