// In progress

use crate::{
    hintengine::rule::Rule,
    model::hints::{PgHint, PgHintList},
    model::postgresplan::{JoinNode, PlanNode, ScanNode},
};
use postgres::Client;
pub struct OrderByIncorrectIndex {
    pg_hint_list: PgHintList,
    conn: Client,
}

impl Rule for OrderByIncorrectIndex {
    fn apply(&mut self, plan: PlanNode) -> Option<PgHintList> {
        // self.apply_recursive(plan);
        Some(self.pg_hint_list.clone())
    }

    fn requires_analyzed_plan(&self) -> bool {
        false
    }
}

impl OrderByIncorrectIndex {
    pub fn new(conn: Client) -> Self {
        Self {
            pg_hint_list: PgHintList::new(),
            conn,
        }
    }

    // fn get_
}
