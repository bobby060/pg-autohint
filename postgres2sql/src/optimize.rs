use crate::connector::query_to_plan;
use crate::hints::{PgHint, PgHintList};
use crate::postgresplan::PlanRoot;
use crate::rule::*;
use crate::rules::*; // Import NljToHashJoin
use postgres::Client;

pub struct Optimizer {
    pub rules: Vec<Box<dyn Rule>>,
}

impl Optimizer {
    pub fn new() -> Self {
        Optimizer { rules: vec![] }
    }

    /// optimize a given sql query by running EXPLAIN / EXPLAIN ANALYZE with the provided connection
    /// returns an optimized SQL with hints
    pub fn optimize(&mut self, sql: &str, conn: &mut Client) -> Result<String, String> {
        let plan = query_to_plan(sql, conn, true);
        let plan_root = plan[0].clone();
        let hints: PgHintList = self.optimize_plan(plan_root);

        Ok(hints.with_sql(sql))
    }

    /// add an instances of crate::rule::* to be applied
    pub fn add_rule(&mut self, rule: Box<dyn Rule>) {
        self.rules.push(rule);
    }

    /// apply rules in self.rules to the plan nodes, will check if plan is analyzed
    /// if plan is not analyzed, will only apply rules with requires_analyzed_plan == false
    fn optimize_plan(&mut self, plan: PlanRoot) -> PgHintList {
        let is_analyzed = plan.is_analyzed();
        let mut pg_hint_list = PgHintList::new();
        // Apply each rule to plan wrapper
        for rule in &mut self.rules {
            if !is_analyzed && rule.requires_analyzed_plan() {
                continue;
            }
            match rule.apply(plan.plan.clone()) {
                Some(hints) => {
                    pg_hint_list.add_hint_list(hints);
                }
                None => {
                    continue;
                }
            }
        }
        pg_hint_list
    }
}

#[cfg(test)]
mod test_optimizer {
    use crate::postgresplan::{postgres2plan, postgres2planroot};

    use super::*;

    /// test NljToHashjoin rule on two NLJs
    #[test]
    fn test_nlj_to_hashjoin() {
        let nlj_to_hashjoin_rule = NljToHashJoin::new(1.2, 1000);

        // in this test, one NLJ has very large plan rows and one have actual rows larger than plan rows
        // expected behavior is two hashjoin hints
        let input_path = "resources/test_json/nlj_rule_test.json";
        let input = std::fs::read_to_string(input_path).expect("Failed to read input file");
        let plan_node = postgres2planroot(&input).unwrap();

        let original_query = std::fs::read_to_string("resources/test_sql/nlj_rule_test.sql")
            .expect("Failed to read input file");

        let mut optimizer = Optimizer::new();
        optimizer.add_rule(Box::new(nlj_to_hashjoin_rule));

        let hint_list = optimizer.optimize_plan(plan_node);
        assert_eq!(
            hint_list.size(),
            2,
            "{}",
            format!("expected 2 hashjoin hints, got {}", hint_list.size())
        );
        println!("{}", hint_list.with_sql(&original_query));
    }
}
