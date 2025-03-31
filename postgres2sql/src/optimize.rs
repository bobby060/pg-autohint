use crate::connector::query_to_plan;
use crate::hints::{PgHint, PgHintList};
use crate::postgresplan::PlanRoot;
use crate::rules::*;
use postgres::Client;

pub struct Optimizer {
    pub rules: Vec<Box<dyn Rule>>,
}

impl Optimizer {
    pub fn new() -> Self {
        Optimizer { rules: vec![] }
    }

    pub fn optimize(&self, sql: &str, conn: &mut Client) -> Result<String, String> {
        let plan = query_to_plan(sql, conn);

        let hints = self.optimize_plan(plan[0].clone());

        Ok(hints.with_sql(sql))
    }

    fn optimize_plan(&self, plan: PlanRoot) -> PgHintList {
        todo!("Implement optimize logic")

        // Apply each rule to plan wrapper
    }
}
