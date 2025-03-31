struct NljToHashJoin;

impl Rule for NljToHashJoin {
    fn apply(&self, plan: PlanNode) -> Option<PgHintList> {
        let hint_list = PgHintList::new();

        for child in plan.children {
            let child_hint_list = self.apply(child);

            if let Some(child_hint_list) = child_hint_list {
                hint_list.add_hint_list(child_hint_list);
            }
        }

        if let PlanNode::NestedLoopJoin(nlj) = plan {
            // hint_list.add_hint(PgHint::HashJoin()));
        }

        // TODO: remove redundant hints (e.g. HashJoin(A) and HashJoin(A) are both in list (is in list twice))

        // TODO: Add selectivity check

        Some(hint_list)
    }
}

#[cfg(test)]
mod test_example_rule {
    use super::*;

    #[test]
    fn test_apply() {
        let rule = NljToHashJoin;
    }
}
