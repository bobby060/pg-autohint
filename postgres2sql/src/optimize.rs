use crate::connector::query_to_plan;
use crate::hints::PgHintList;
use crate::postgresplan::PlanRoot;
use crate::rule::*;
use postgres::Client;

/// Optimizer struct
///
/// # Fields
///
/// * `rules`: A vector of rules to apply to the plan
pub struct Optimizer {
    pub rules: Vec<Box<dyn Rule>>,
}

impl Optimizer {
    pub fn new() -> Self {
        Optimizer { rules: vec![] }
    }

    /// optimize a given sql query by running EXPLAIN / EXPLAIN ANALYZE with the provided connection
    /// returns an optimized SQL with hints
    ///
    /// # Arguments
    ///
    /// * `sql`: The SQL query to optimize
    /// * `conn`: The connection to the database
    ///
    /// # Returns String
    pub fn optimize(
        &mut self,
        sql: &str,
        conn: &mut Client,
        is_analyze: bool,
    ) -> Result<String, String> {
        let plan = query_to_plan(sql, conn, is_analyze);
        let plan_root = plan.to_owned();
        let hints: PgHintList = self.optimize_plan(plan_root);

        Ok(hints.with_sql(sql))
    }

    /// add an instances of rule to be applied
    ///
    /// # Arguments
    ///
    /// * `rule`: The rule to add
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
    use crate::connector::establish_connection;
    use crate::postgresplan::postgres2planroot;

    use super::*;
    use crate::rules::{CardCorrection, NljToHashJoin}; // Import NljToHashJoin

    /// test NljToHashjoin rule on two NLJs
    #[test]
    fn test_optimize_plan() {
        // TODO: config this rule so it picks nlj for outer join but hash for inner join
        // TODO: figure out how parallel works in EXPLAIN ANALYZE, it is showing 0 rows out of name_basics but there are should be 5 actually
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

    #[test]
    fn test_optimize() {
        let rule = NljToHashJoin::new(1.2, 1000);
        let mut optimizer = Optimizer::new();
        optimizer.add_rule(Box::new(rule));

        let sql = "SELECT * FROM title_basics";
        let mut conn = establish_connection("imdb", "postgres", "postgres", "localhost", "5432");
        let new_sql = optimizer.optimize(sql, &mut conn, false);

        assert_eq!(new_sql.unwrap(), sql);
    }

    /// test CardCorrection rule on two NLJs
    #[test]
    fn test_card_correction() {
        let card_correction_rule = CardCorrection::new(1.2);

        // in this test, one NLJ has very large plan rows and one have actual rows larger than plan rows
        // expected behavior is two hashjoin hints
        let input_path = "resources/test_json/card_correction_rule_test.json";
        let input = std::fs::read_to_string(input_path).expect("Failed to read input file");
        let plan_node = postgres2planroot(&input).unwrap();

        let original_query =
            std::fs::read_to_string("resources/test_sql/card_correction_rule_test.sql")
                .expect("Failed to read input file");

        let mut optimizer = Optimizer::new();
        optimizer.add_rule(Box::new(card_correction_rule));

        let hint_list = optimizer.optimize_plan(plan_node);
        assert_eq!(
            hint_list.size(),
            2,
            "{}",
            format!("expected 2 card correction hints, got {}", hint_list.size())
        );
        println!("{}", hint_list.with_sql(&original_query));
    }
}
