use crate::{
    hintengine::rule::Rule,
    model::hints::{PgHint, PgHintList},
    model::postgresplan::{JoinNode, PlanNode, ScanNode},
};

/// NljToHashJoin rule that requires analyze
pub struct NljToHashJoin {
    actual_plan_ratio: f32,
    plan_rows_threshold: i64,
    pg_hint_list: PgHintList,
}

impl Rule for NljToHashJoin {
    /// When actual cost is double
    fn apply(&mut self, plan: PlanNode) -> Option<PgHintList> {
        // TODO: remove redundant hints (e.g. HashJoin(A) and HashJoin(A) are both in list (is in list twice))

        // TODO: Add selectivity check

        self.apply_recursive(plan);

        Some(self.pg_hint_list.clone())
    }

    fn requires_analyzed_plan(&self) -> bool {
        true
    }
}

impl NljToHashJoin {
    /// Creates a new instance of `NljToHashJoin`.
    ///
    /// # Arguments
    ///
    /// * `rows_threshold` - An integer representing the threshold for the number of rows.
    ///   If the `Actual Rows` exceeds the `Plan Rows` by or equal to this threshold, the conversion
    ///   from Nested Loop Join (NLJ) to Hash Join will be triggered.
    ///
    /// # Returns
    ///
    /// A new `NljToHashJoin` instance with the specified `rows_threshold` and an
    /// initialized `PgHintList`.
    pub fn new(actual_plan_ratio: f32, plan_rows_threshold: i64) -> Self {
        NljToHashJoin {
            actual_plan_ratio: actual_plan_ratio,
            plan_rows_threshold: plan_rows_threshold,
            pg_hint_list: PgHintList::new(),
        }
    }

    /// heuristic check whether the nlj should be converted into a hashjoin
    fn nlj_to_hash_cond(&self, plan: JoinNode) -> bool {
        match plan {
            JoinNode::NestedLoopJoin(nlj) => {
                // TODO:
                let actual_rows = nlj.actual_rows.unwrap_or_else(|| panic!("actual_rows not parsed. NljToHashJoin should only apply under ANALYZE mode"));
                let plan_rows = nlj
                    .plan_rows
                    .unwrap_or_else(|| panic!("plan_rows not parsed"));
                if plan_rows > self.plan_rows_threshold {
                    return true;
                }
                if actual_rows as f32 >= plan_rows as f32 * self.actual_plan_ratio {
                    return true;
                }
                false
            }
            _ => false,
        }
    }

    /// function that applies the heuristic check on a NLJ node,
    /// if condition is met, add a hashjoin hint to self.pg_hint_list
    fn nlj_to_hash(&mut self, plan: JoinNode, joins: String) {
        if self.nlj_to_hash_cond(plan) {
            self.pg_hint_list
                .add_hint(PgHint::HashJoin { tables: joins });
        }
    }

    /// visit tree and while getting the join relations bottom up
    /// call to get_join and get_scan methods correspondingly
    /// that adds join hint to convert NLJ to HashJoin if condition is met
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
                    None => "".to_string()
                }
            }
        }
    }

    /// add join method hint to self.pg_hint_list if the heuristic condition is met.
    /// returns the tables joined together to facilitate hinting of joins above
    fn get_apply_join(&mut self, join_node: JoinNode) -> String {
        let left = self.apply_recursive(join_node.get_left().unwrap());
        let right = self.apply_recursive(join_node.get_right().unwrap());
        if left.eq("") || right.eq("") {
            return "".to_string();
        }
        let joins = format!("{} {}", left, right);

        self.nlj_to_hash(join_node, joins.clone());

        joins
    }

    /// return the relation name for constructing hints for the joins above
    fn get_scan(&mut self, scan_node: ScanNode) -> String {
        let rel_name = scan_node.get_relation_name();
        rel_name
    }
}

#[cfg(test)]
mod test_nlj_to_hashjoin_rule {
    use super::*;
    use crate::model::postgresplan::PlanRoot;

    #[test]
    fn test_apply() {
        let mut nlj_to_hashjoin_rule = NljToHashJoin::new(1.2, 1000);

        // in this test, one NLJ has very large plan rows and one have actual rows larger than plan rows
        // expected behavior is two hashjoin hints
        let input_path = "resources/test_json/nlj_rule_test.json";
        let plan_node = PlanRoot::from_json(input_path).unwrap();

        let hint_list = nlj_to_hashjoin_rule.apply(plan_node.plan).unwrap();
        println!("{}", hint_list.to_string());
        assert_eq!(
            hint_list.size(),
            2,
            "{}",
            format!("expected 2 hashjoin hints, got {}", hint_list.size())
        );
    }

    #[test]
    #[ignore] // query takes a very long time to run
    fn nlj_to_hashjoin_rule_test() {
        crate::test_utils::rule_test(
            Box::new(NljToHashJoin::new(1.2, 1000)),
            std::fs::read_to_string("resources/test_sql/nlj_rule_test.sql")
                .unwrap()
                .as_str(),
            true,
        );
    }
}
