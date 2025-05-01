use crate::hintengine::Rule;
use crate::model::{
    hints::PgHintList,
    postgresplan::PlanRoot,
    query::{Query, TimeOut},
};
use postgres::Client;

/// Optimizer struct
///
/// # Fields
///
/// * `rules`: A vector of rules to apply to the plan
pub struct Optimizer {
    pub rules: Vec<Box<dyn Rule>>,
    enable_hint_table: bool,
    default_timeout: Option<TimeOut>,
    hint_table_app_name: String
}

impl Optimizer {
    pub fn new(enable_hint_table: bool, hint_table_app_name: Option<&str>) -> Self {
        Optimizer {
            rules: vec![],
            enable_hint_table: enable_hint_table,
            default_timeout: None,
            hint_table_app_name: hint_table_app_name.unwrap_or_else(|| "").to_string()
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
    ) -> Result<Query, String> {
        let mut query = Query::new(sql.to_string(), None, self.default_timeout.clone());
        let plan = query.get_plan(conn, is_analyze)?;
        let mut query_id = None;
        if self.enable_hint_table {
            // check if hint_plan.hints table is created, init hint table if not
            if conn
                .query("SELECT 1 FROM pg_catalog.pg_tables WHERE schemaname='hint_plan' AND tablename = 'hints'", &[])
                .unwrap()
                .is_empty()
            {
                println!("Info: The hint_plan.hints table does not exist. Creating new hint table.");
                self.init_hint_table(conn);
            } else {
                println!("Info: The hint_plan.hints table exists. Reusing existing hint table.");
            }
            conn.execute("SET pg_hint_plan.enable_hint_table='on'", &[])
                .unwrap();
            query_id = plan.query_id.clone();
        } else {
            conn.execute("SET pg_hint_plan.enable_hint_table='off'", &[])
                .unwrap();
        }
        let hints: PgHintList = self.apply_rules(plan);
        if self.enable_hint_table {
            match query_id {
                Some(id) => {
                    let hint_table_app_name = self.hint_table_app_name.clone();
                    self.add_hint_table(conn, id, &hint_table_app_name, &&hints.to_hint_table_string());
                }
                None => {
                    // if query_id is None, send a warning that hint table will not be used because no query id is parsed
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

    /// clear all hints in the hint table hint_plan.hints
    ///
    /// # Arguments
    ///
    /// * `conn`: The postgres connection
    pub fn init_hint_table(&mut self, conn: &mut Client) {
        // conn.execute("TRUNCATE TABLE IF EXISTS hint_plan.hints", &[])
        // .unwrap_or_else(|e| {
        //     eprintln!("Failed to truncate hint_plan.hints table: {}", e.to_string());
        //     0
        // });
        conn.execute("SET pg_hint_plan.enable_hint_table='off'", &[])
            .unwrap_or_else(|e| {
                eprintln!("Failed to drop pg_hint_plan extension: {}", e.to_string());
                0
            });
        conn.execute("DROP EXTENSION IF EXISTS pg_hint_plan CASCADE", &[])
            .unwrap_or_else(|e| {
                eprintln!("Failed to drop pg_hint_plan extension: {}", e.to_string());
                0
            });
        conn.execute("CREATE EXTENSION IF NOT EXISTS pg_hint_plan", &[])
            .unwrap_or_else(|e| {
                eprintln!("Failed to create pg_hint_plan extension: {}", e.to_string());
                0
            });
        conn.execute("SET pg_hint_plan.enable_hint_table='on'", &[])
            .unwrap_or_else(|e| {
                eprintln!("Failed to drop pg_hint_plan extension: {}", e.to_string());
                0
            });
    }

    /// add a hint to the hint table
    ///
    /// # Arguments
    ///
    /// * `query_id`: the query identifier obtained via `EXPLAIN VERBOSE`
    /// * `hints`: The hints to add as String
    /// * `application_name`: optional, the value of application_name where sessions can apply a hint. if not specified, hint will apply to all applications
    fn add_hint_table(
        &mut self,
        conn: &mut Client,
        query_id: i64,
        application_name: &str,
        hints: &str,
    ) {
        if hints.is_empty() {
            return;
        }
        let query = self.insert_hint_to_hint_table(query_id, application_name, hints);
        // TODO: fix when hint is already there!
        conn.execute(&query, &[]).unwrap_or_else(|e| {
            eprintln!("Failed to insert into hint table: {}", e.to_string());
            0
        });
    }

    fn insert_hint_to_hint_table(
        &mut self,
        query_id: i64,
        application_name: &str,
        hints: &str,
    ) -> String {
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

        self.reset_rules();
        pg_hint_list
    }

    fn reset_rules(&mut self) {
        for rule in &mut self.rules {
            rule.reset();
        }
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

        let mut optimizer = Optimizer::new(false, None);
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
        let mut optimizer = Optimizer::new(false, None);
        optimizer.add_rule(Box::new(rule));

        let sql = "SELECT * FROM title_basics";
        let mut conn = establish_connection("imdb", "postgres", "postgres", "localhost", "5432");
        let new_sql = optimizer.optimize(sql, &mut conn, false);

        assert_eq!(new_sql.unwrap().get_original_sql(), sql);
    }

    /// test CardCorrection rule on two NLJs
    #[test]
    fn test_card_correction() {
        let card_correction_rule = CardCorrection::new();

        // in this test, one NLJ has very large plan rows and one have actual rows larger than plan rows
        // expected behavior is two hashjoin hints
        let input_path = "resources/test_json/card_correction_rule_test.json";
        let plan_node = PlanRoot::from_json(input_path).unwrap();

        let original_query =
            std::fs::read_to_string("resources/test_sql/card_correction_rule_test.sql")
                .expect("Failed to read input file");

        let mut optimizer = Optimizer::new(false, None);
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
        let mut optimizer = Optimizer::new(true, None);
        let query_id = -7164653396197960701;
        let application_name = "";
        let hints = "SeqScan(t1)";
        assert!(optimizer.insert_hint_to_hint_table(query_id, application_name, hints)
        .eq("INSERT INTO hint_plan.hints(query_id, application_name, hints) VALUES (-7164653396197960701, '', 'SeqScan(t1)');"));
    }

    #[test]
    fn test_run_hint_table_query() {
        let mut optimizer = Optimizer::new(true, None);
        optimizer.add_rule(Box::new(NljToHashJoin::new(1.0, 1000)));
        let mut conn = establish_connection("imdb", "postgres", "postgres", "localhost", "5432");
        let sql = std::fs::read_to_string("resources/test_sql/nlj_rule_test.sql")
            .expect("Failed to read input file");
        // insert hints into hint table
        let result = optimizer
            .optimize(&sql, &mut conn, true)
            .unwrap();
        assert_eq!(
            result.get_hints().unwrap().size(),
            2,
            "expected 2 hints, got {}",
            result.get_hints().unwrap().size()
        );
        // run query again, should have hints applied
        let query = Query::new(sql.to_string(), None, optimizer.default_timeout.clone());
        let plan = query.get_plan(&mut conn, false).unwrap();
        // assert HashJoin is in plan
        assert!(
            format!("{:#?}", plan.plan).contains("HashJoin"),
            "Expected HashJoin in the plan, but it was not found. Plan: {:#?}",
            plan
        );
    }


    #[test]
    fn test_run_hint_table_query_cold_start() {
        // reset hint table and disable it for testing
        let mut conn = establish_connection("imdb", "postgres", "postgres", "localhost", "5432");
        conn.execute("ALTER SYSTEM SET pg_hint_plan.enable_hint_table = 'off'", &[]).unwrap();
        conn.execute("DROP EXTENSION IF EXISTS pg_hint_plan", &[]).unwrap();

        let mut optimizer = Optimizer::new(true, None);
        optimizer.add_rule(Box::new(NljToHashJoin::new(1.0, 1000)));
        let sql = std::fs::read_to_string("resources/test_sql/nlj_rule_test.sql")
            .expect("Failed to read input file");
        // insert hints into hint table
        let result = optimizer
            .optimize(&sql, &mut conn, true)
            .unwrap();
        assert_eq!(
            result.get_hints().unwrap().size(),
            2,
            "expected 2 hints, got {}",
            result.get_hints().unwrap().size()
        );
        // run query again, should have hints applied
        let query = Query::new(sql.to_string(), None, optimizer.default_timeout.clone());
        let plan = query.get_plan(&mut conn, false).unwrap();
        // assert HashJoin is in plan
        assert!(
            format!("{:#?}", plan.plan).contains("HashJoin"),
            "Expected HashJoin in the plan, but it was not found. Plan: {:#?}",
            plan
        );
    }
}
