use crate::hints::PgHintList;
use crate::postgresplan::PlanNode;

pub trait Rule {
    fn apply(&mut self, plan: PlanNode) -> Option<PgHintList>;

    fn requires_analyzed_plan(&self) -> bool;
}
