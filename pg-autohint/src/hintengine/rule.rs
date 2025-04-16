use crate::model::hints::PgHintList;
use crate::model::postgresplan::PlanNode;

/// Trait each rule must implement
pub trait Rule {
    /// Apply the rule to the plan and output any hints resulting from the rule
    fn apply(&mut self, plan: PlanNode) -> Option<PgHintList>;

    /// Whether the rule requires an analyzed plan to be applied
    fn requires_analyzed_plan(&self) -> bool;
}
