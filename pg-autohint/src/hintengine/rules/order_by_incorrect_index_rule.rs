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
use crate::{
    hintengine::rule::Rule,
    model::catalog::Catalog,
    model::hints::{PgHint, PgHintList},
    model::postgresplan::PlanNode,
};
use postgres::Client;
pub struct OrderByIncorrectIndex {
    pg_hint_list: PgHintList,
    catalog: Catalog,
}

impl Rule for OrderByIncorrectIndex {
    fn apply(&mut self, plan: PlanNode) -> Option<PgHintList> {
        if let PlanNode::IndexScan(index_scan) = plan {
            if let Some(filter) = index_scan.filter {
                if !filter.contains(
                    &self
                        .catalog
                        .indexes
                        .get(&index_scan.index_name)
                        .unwrap()
                        .columns[0], // Only check the first column of the index
                ) {
                    let table = self.catalog.tables.get(&index_scan.relation_name.unwrap());
                    if let Some(table) = table {
                        for index_name in table.indexes.iter() {
                            let index = self.catalog.indexes.get(index_name).unwrap();
                            if index.columns.len() == 1 && filter.contains(&index.columns[0]) {
                                let mut hints = PgHintList::new();
                                hints.add_hint(PgHint::IndexScan {
                                    table: index.table_name.clone(),
                                    index: Some(index_name.clone()),
                                });
                                return Some(hints);
                            }
                        }
                    }
                    None
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            let mut hints = PgHintList::new();
            for child in plan.get_children().unwrap() {
                hints.concat_hint_list(self.apply(child).unwrap_or_else(|| PgHintList::new()));
            }
            if hints.size() == 0 {
                None
            } else {
                Some(hints)
            }
        }
    }

    fn requires_analyzed_plan(&self) -> bool {
        false
    }

    fn reset(&mut self) {
        self.pg_hint_list = PgHintList::new();
    }
}

impl OrderByIncorrectIndex {
    pub fn new(conn: &mut Client) -> Self {
        Self {
            pg_hint_list: PgHintList::new(),
            catalog: Catalog::new(conn),
        }
    }
}

#[cfg(test)]
mod test_order_by_incorrect_index_rule {
    use super::*;
    use crate::test_utils::get_test_connection;
    #[test]
    fn test_order_by_incorrect_index_rule() {
        let mut conn = get_test_connection();

        let sql = "SELECT
        FROM orders_test
        WHERE TRUE
        AND shipping_date >= '2022-05-01'
        AND shipping_date <= '2022-05-01'
        ORDER BY order_id
        LIMIT 50;";
        let hints = crate::test_utils::rule_test(
            Box::new(OrderByIncorrectIndex::new(&mut conn)),
            sql,
            false,
        );
        // assert_eq!(hints.size(), 1);
        // assert_eq!(hints.to_string().contains("IndexScan"), true,);
    }
}
