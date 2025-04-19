use crate::hintengine::Rule;
use crate::model::{
    hints::PgHintList,
    postgresplan::PlanRoot,
    query::{Query, TimeOut},
};
use postgres::{Client, GenericClient};

/// Optimizer struct
///
/// # Fields
///
/// * `rules`: A vector of rules to apply to the plan
pub struct Optimizer {
    pub rules: Vec<Box<dyn Rule>>,
    default_timeout: Option<TimeOut>,
}

impl Optimizer {
    pub fn new() -> Self {
        Optimizer {
            rules: vec![],
            default_timeout: None,
        }
    }

    /// optimize a given sql query by running EXPLAIN / EXPLAIN ANALYZE with the provided connection
    /// returns an optimized SQL with hints
    ///
    /// # Arguments
    ///
    /// * `sql`: The SQL query to optimize
    /// * `conn`: The connection to the database
    /// * `is_analyze`: Whether the query would be executed via EXPLAIN ANALYZE
    /// * `add_hint_table`: Whether the generated hints should be added to hint table
    /// * `hint_table_app_name`: only used when `add_hint_table` is true. optional, the value of application_name where sessions can apply a hint. 
    /// if not specified, hint will enabled to all applications
    ///
    /// # Returns
    ///
    /// A `Query` object representing the optimized SQL query with hints
    pub fn optimize(
        &mut self,
        sql: &str,
        conn: &mut Client,
        is_analyze: bool,
        add_hint_table: bool,
        hint_table_app_name: Option<&str>,
    ) -> Result<Query, String> {
        let mut query = Query::new(sql.to_string(), None, self.default_timeout.clone());
        let plan = query.get_plan(conn, is_analyze)?;
        let mut query_id = None;
        if add_hint_table {
            // if query_id is None, send a warning that hint table will not be used because no query id is parsed
            query_id = plan.query_id.clone();
        }
        let hints: PgHintList = self.apply_rules(plan);
        if add_hint_table {
            match query_id {
                Some(id) => {
                    let application_name = hint_table_app_name.unwrap_or_else(|| "");
                    self.add_hint_table(conn, id, application_name, &hints.to_string());
                }
                None => {
                    eprint!("Warning: Hint table will not be used for query because no query ID is parsed: {}.\n
                    consider running SET compute_query_id = 'on'; ", sql);
                }
            }
        }
        query.add_hint_list(hints);

        Ok(query)
    }

    /// add a rule to the optimizer
    ///
    /// # Arguments
    ///
    /// * `rule`: The rule to add
    pub fn add_rule(&mut self, rule: Box<dyn Rule>) {
        self.rules.push(rule);
    }

    /// add a hint to the hint table
    /// 
    /// # Arguments
    /// 
    /// * `query_id`: the query identifier obtained via `EXPLAIN VERBOSE`
    /// * `hints`: The hints to add as String
    /// * `application_name`: optional, the value of application_name where sessions can apply a hint. if not specified, hint will apply to all applications
    fn add_hint_table(&mut self, conn: &mut Client, query_id: i64, application_name: &str, hints: &str) {
        let query = self.add_hint_table_query(query_id, application_name, hints);
        conn.execute(&query, &[]).unwrap_or_else(|e| {
            eprintln!("Failed to insert into hint table: {}", e.to_string());
            0
        });
    }

    fn add_hint_table_query(&mut self, query_id: i64, application_name: &str, hints: &str) -> String {
        format!("INSERT INTO hint_plan.hints(query_id, application_name, hints) VALUES ({}, '{}', '{}');", query_id, application_name, hints)
    }

    /// apply rules in self.rules to the plan nodes, will check if plan is analyzed
    /// if plan is not analyzed, will only apply rules with requires_analyzed_plan == false
    fn apply_rules(&mut self, plan: PlanRoot) -> PgHintList {
        let is_analyzed = plan.is_analyzed();
        let mut pg_hint_list = PgHintList::new();
        // Apply each rule to plan wrapper
        for rule in &mut self.rules {
            if !is_analyzed && rule.requires_analyzed_plan() {
                continue;
            }
            match rule.apply(plan.plan.clone()) {
                Some(hints) => {
                    pg_hint_list.concat_hint_list(hints);
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
    use crate::model::postgresplan::PlanRoot;

    use super::*;
    use crate::hintengine::rules::{CardCorrection, NljToHashJoin}; // Import NljToHashJoin

    /// test NljToHashjoin rule on two NLJs
    #[test]
    fn test_optimize_plan() {
        // TODO: config this rule so it picks nlj for outer join but hash for inner join
        // TODO: figure out how parallel works in EXPLAIN ANALYZE, it is showing 0 rows out of name_basics but there are should be 5 actually
        let nlj_to_hashjoin_rule = NljToHashJoin::new(1.2, 1000);

        // in this test, one NLJ has very large plan rows and one have actual rows larger than plan rows
        // expected behavior is two hashjoin hints
        let input_path = "resources/test_json/nlj_rule_test.json";
        let plan_node = PlanRoot::from_json(input_path).unwrap();

        let original_query = std::fs::read_to_string("resources/test_sql/nlj_rule_test.sql")
            .expect("Failed to read input file");

        let mut optimizer = Optimizer::new();
        optimizer.add_rule(Box::new(nlj_to_hashjoin_rule));

        let hint_list = optimizer.apply_rules(plan_node);
        assert_eq!(
            hint_list.size(),
            2,
            "{}",
            format!("expected 2 hashjoin hints, got {}", hint_list.size())
        );

        let query = Query::new(original_query, Some(hint_list), None);
        println!("{}", query);
    }

    #[test]
    fn test_optimize() {
        let rule = NljToHashJoin::new(1.2, 1000);
        let mut optimizer = Optimizer::new();
        optimizer.add_rule(Box::new(rule));

        let sql = "SELECT * FROM title_basics";
        let mut conn = establish_connection("imdb", "postgres", "postgres", "localhost", "5432");
        let new_sql = optimizer.optimize(sql, &mut conn, false, false, None);

        assert_eq!(new_sql.unwrap().get_original_sql(), sql);
    }

    /// test CardCorrection rule on two NLJs
    #[test]
    fn test_card_correction() {
        let card_correction_rule = CardCorrection::new(1.0);

        // in this test, one NLJ has very large plan rows and one have actual rows larger than plan rows
        // expected behavior is two hashjoin hints
        let input_path = "resources/test_json/card_correction_rule_test.json";
        let plan_node = PlanRoot::from_json(input_path).unwrap();

        let original_query =
            std::fs::read_to_string("resources/test_sql/card_correction_rule_test.sql")
                .expect("Failed to read input file");

        let mut optimizer = Optimizer::new();
        optimizer.add_rule(Box::new(card_correction_rule));

        let hint_list = optimizer.apply_rules(plan_node);
        assert_eq!(
            hint_list.size(),
            2,
            "{}",
            format!("expected 2 card correction hints, got {}", hint_list.size())
        );
        let query = Query::new(original_query, Some(hint_list), None);
        println!("{}", query);
    }

    #[test]
    fn test_add_hint_table_query() {
        let mut optimizer = Optimizer::new();
        let query_id = -7164653396197960701;
        let application_name = "";
        let hints = "SeqScan(t1)";
        assert!(optimizer.add_hint_table_query(query_id, application_name, hints)
        .eq("INSERT INTO hint_plan.hints(query_id, application_name, hints) VALUES (-7164653396197960701, '', 'SeqScan(t1)');"))
    }
}



