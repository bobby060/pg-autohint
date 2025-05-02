<<<<<<< Updated upstream
// In progress

// General idea: if we a predicate on a column that HAS an index, but the index is not used, use it!

=======
/// Scenario:
/// When a table is indexed on two columns: the predicate column and the order by column,
/// Sometimes, Postgres will choose the order by index incorrectly. When this occurs, we
///
///
/// Example query for this scenario:
/// EXPLAIN ANALYZE SELECT
/// FROM orders_test
/// WHERE TRUE
/// AND shipping_date >= '2022-05-01'
/// AND shipping_date <= '2022-05-01'
/// ORDER BY order_id
/// LIMIT 50;
///
///
>>>>>>> Stashed changes
use crate::{
    hintengine::rule::Rule,
    model::catalog::Catalog,
    model::hints::{PgHint, PgHintList},
    model::postgresplan::{JoinNode, PlanNode, ScanNode},
    model::catalog::Catalog,
};
use postgres::Client;
pub struct OrderByIncorrectIndex {
    pg_hint_list: PgHintList,
    conn: Client,
    catalog: Catalog,
}

struct AccessedTable {
    name: String,
    columnsAccessed: Vec<String>,
    columnsUsedInOrderBy: Vec<String>,
}

impl Rule for OrderByIncorrectIndex {
    fn apply(&mut self, plan: PlanNode) -> Option<PgHintList> {
<<<<<<< Updated upstream
        let catalog = Catalog::new(&mut self.conn);

        let mut hint_list = PgHintList::new();

        // let mut
=======
        let 
        // Base case: index scan
>>>>>>> Stashed changes

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
            catalog: Catalog::new(conn),
        }
    }

    fn apply_recursive(&mut self, plan: PlanNode) -> Option<PgHintList> {
        if let PlanNode::IndexScan(index_scan) = plan {
            if index_scan.index_name.is_some() {
                // let index_name = index_scan.index_name.unwrap();
                // let index_def = self
                //     .conn
                //     .query_one(
                //         "SELECT indexdef FROM pg_indexes WHERE tablename = $1 AND indexname = $2",
                //         &[&index_name],
                //     )
                //     .unwrap();
            }
        } else {
            let mut hints = PgHintList::new();
            for child in plan.children() {
                hints.concat_hint_list(self.apply_recursive(child));
            }
            if hints.empty() {
                None
            } else {
                Some(hints)
            }
        }
    }

    // fn get_
}
