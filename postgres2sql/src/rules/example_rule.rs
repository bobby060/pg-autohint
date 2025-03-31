struct NljToHashJoin;

impl Rule for NljToHashJoin {
    fn apply(&self, plan: PlanNode) -> Option<PgHintList> {
        if let PlanNode::NestedLoopJoin(nlj) = plan {
            Some(PgHint::HashJoin(nlj.hash_join()))
        } else {
        }
    }
}

#[cfg(test)]
mod test_example_rule {
    use super::*;

    #[test]
    fn test_apply() {
        let rule = ExampleRule;
    }
}
