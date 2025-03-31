use crate::hints::{PgHint, PgHintList};
use crate::postgresplan::PlanNode;

pub trait Rule {
    fn apply(&self, plan: PlanNode) -> Option<PgHintList>;

    fn requires_analyzed_plan(&self) -> bool;
}
