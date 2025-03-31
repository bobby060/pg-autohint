use crate::hints::PgHint;
use crate::rules::*;

struct Optimizer {
    rules: Vec<Box<dyn Rule>>,
}

impl Optimizer {
    pub fn new() -> Self {
        Optimizer { rules: vec![] }
    }

    pub fn optimize(&self, sql: &str, conn: &mut Client) -> Result<String, String> {
        let plan = query_to_plan(sql, conn);

        let hints = self.optimize_plan(plan);

        Ok(hints.with_sql(sql))
    }

    fn optimize_plan(&self, plan: PlanWrapper) -> PgHintList {
        todo!("Implement optimize logic")

        // Apply each rule to plan wrapper
    }
}
