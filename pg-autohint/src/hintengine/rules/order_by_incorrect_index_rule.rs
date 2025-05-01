// In progress

// General idea: if we a predicate on a column that HAS an index, but the index is not used, use it!

use crate::{
    hintengine::rule::Rule,
    model::catalog::Catalog,
    model::hints::{PgHint, PgHintList},
    model::postgresplan::{JoinNode, PlanNode, ScanNode},
};
use postgres::Client;
pub struct OrderByIncorrectIndex {
    pg_hint_list: PgHintList,
    conn: Client,
}

struct AccessedTable {
    name: String,
    columnsAccessed: Vec<String>,
    columnsUsedInOrderBy: Vec<String>,
}

impl Rule for OrderByIncorrectIndex {
    fn apply(&mut self, plan: PlanNode) -> Option<PgHintList> {
        let catalog = Catalog::new(&mut self.conn);

        let mut hint_list = PgHintList::new();

        // let mut

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
