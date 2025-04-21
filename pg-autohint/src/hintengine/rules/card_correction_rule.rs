use crate::{
    hintengine::rule::Rule,
    model::{
        hints::PgHint,
        hints::PgHintList,
        postgresplan::{JoinNode, PlanNode, ScanNode},
    },
};

/// NljToHashJoin rule that requires analyze
pub struct CardCorrection {
    card_multiplier: f64,
    pg_hint_list: PgHintList,
}

impl Rule for CardCorrection {
    fn apply(&mut self, plan: PlanNode) -> Option<PgHintList> {
        self.apply_recursive(plan);
        Some(self.pg_hint_list.clone())
    }

    fn requires_analyzed_plan(&self) -> bool {
        true
    }
}

impl CardCorrection {
    pub fn new(card_multiplier: f64) -> Self {
        CardCorrection {
            card_multiplier: card_multiplier,
            pg_hint_list: PgHintList::new(),
        }
    }

    // TODO: the actual cardinality of join is one in the `Gather` node above it. Parallel execution causes the
    // actual_rows in join nodes to be low
    // Ad hoc solution is to multiply card by self.card_multiplier that is multiplied by the number of workers launched
    // in Gather nodes
    fn correct_join_card(&mut self, join_node: JoinNode, joins: String) {
        let actual_rows = join_node.get_card();
        let actual_rows = actual_rows.unwrap();
        let actual_rows = if actual_rows == 0 { 1 } else { actual_rows };

        let adjust_rows = (actual_rows as f64 * self.card_multiplier) as i64;
        self.pg_hint_list.add_hint(PgHint::CardCorrection {
            tables: joins,
            card: adjust_rows,
        });
    }

    fn apply_recursive(&mut self, plan: PlanNode) -> String {
        match plan {
            PlanNode::HashJoin(hash_join) => self.get_apply_join(JoinNode::HashJoin(hash_join)),
            PlanNode::MergeJoin(merge_join) => self.get_apply_join(JoinNode::MergeJoin(merge_join)),
            PlanNode::NestedLoopJoin(nested_loop_join) => {
                self.get_apply_join(JoinNode::NestedLoopJoin(nested_loop_join))
            }
            PlanNode::SeqScan(seq_scan) => self.get_scan(ScanNode::SeqScan(seq_scan)),
            PlanNode::IndexScan(index_scan) => self.get_scan(ScanNode::IndexScan(index_scan)),
            PlanNode::IndexOnlyScan(index_only_scan) => {
                self.get_scan(ScanNode::IndexOnlyScan(index_only_scan))
            }
            PlanNode::ValueScan(value_scan) => self.get_scan(ScanNode::ValueScan(value_scan)),
            PlanNode::BitmapHeapScan(bitmap_heap_scan) => self.get_scan(ScanNode::BitmapHeapScan(bitmap_heap_scan)),
            PlanNode::SampleScan(sample_scan) => self.get_scan(ScanNode::SampleScan(sample_scan)),
            PlanNode::WorkTableScan(work_table_scan) => self.get_scan(ScanNode::WorkTableScan(work_table_scan)),
            PlanNode::CteScan(cte_scan) => self.get_scan(ScanNode::CteScan(cte_scan)),
            PlanNode::FunctionScan(function_scan) => self.get_scan(ScanNode::FunctionScan(function_scan)),
            _ => {
                // if not join or scan nodes, visit children
                let children = plan.get_children();
                match children {
                    Some(children) => match children.len() {
                        1 => self.apply_recursive(children.get(0).unwrap().clone()),
                        _ => {
                            todo!("handle subqueries where a node like Filter could have 2 child, one for input one for subquery as predicate, 
                                  also need to handle SetOps nodes that can have two children, that's technically also subqueries")
                        }
                    },
                    None => panic!("unreachable"),
                }
            }
        }
    }

    /// add join method hint to self.pg_hint_list if the heuristic condition is met.
    /// returns the tables joined together to facilitate hinting of joins above
    fn get_apply_join(&mut self, join_node: JoinNode) -> String {
        let left = self.apply_recursive(join_node.get_left().unwrap());
        let right = self.apply_recursive(join_node.get_right().unwrap());
        let joins = format!("{} {}", left, right);

        self.correct_join_card(join_node, joins.clone());

        joins
    }

    /// return the alias for constructing hints for the joins above,
    /// which can be recognized by pg_hint_plan
    fn get_scan(&mut self, scan_node: ScanNode) -> String {
        scan_node.get_alias()
    }
}

#[cfg(test)]
mod test_card_correction_rule {
    use super::*;
    use crate::model::postgresplan::PlanRoot;

    #[test]
    fn test_apply() {
        let mut card_correction_rule = CardCorrection::new(1.0);

        // in this test, one NLJ has very large plan rows and one have actual rows larger than plan rows
        // expected behavior is two hashjoin hints
        let input_path = "resources/test_json/card_correction_rule_test.json";
        let plan_node = PlanRoot::from_json(input_path).unwrap().plan;

        let hint_list = card_correction_rule.apply(plan_node).unwrap();
        println!("{}", hint_list.to_string());
        assert_eq!(
            hint_list.size(),
            2,
            "{}",
            format!("expected 2 card correction hints, got {}", hint_list.size())
        );
    }
}
