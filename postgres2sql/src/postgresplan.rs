/// Represents the different nodes of a Postgres Plan
/// Deserializes a Postgres plan from a JSON string
use serde::*;
use sqlparser::ast::SetOperator;
/// Wrapper for parsing the whole plan json
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlanRoot {
    #[serde(rename = "Plan")]
    pub plan: PlanNode,
    #[serde(rename = "Execution Time")]
    pub execution_time: Option<f64>,
}

impl PlanRoot {
    pub fn is_analyzed(&self) -> bool {
        self.execution_time.is_some()
    }
}

/// Plan node type enum for `serde` json parsing
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "Node Type")]
pub enum PlanNode {
    Aggregate(Aggregate),
    // scans
    #[serde(rename = "Seq Scan")]
    SeqScan(SeqScan),
    #[serde(rename = "Index Scan")]
    IndexScan(IndexScan),
    #[serde(rename = "Values Scan")]
    ValueScan(ValueScan),
    #[serde(rename = "Subquery Scan")]
    SubqueryScan(SubqueryScan),
    // joins
    Hash(Hash),
    #[serde(rename = "Hash Join")]
    HashJoin(HashJoin),
    #[serde(rename = "Merge Join")]
    MergeJoin(MergeJoin),
    // limit, unique, sort
    Limit(Limit),
    Sort(Sort),
    Unique(Unique), // Alias for distinct
    Append(Append), // Alias for Union (we think?)
    Gather(Gather),
    #[serde(rename = "Gather Merge")]
    GatherMerge(GatherMerge),
    #[serde(rename = "Nested Loop")]
    NestedLoopJoin(NestedLoopJoin),
    #[serde(rename = "Index Only Scan")]
    IndexOnlyScan(IndexOnlyScan),
    Memoize(Memoize),
    #[serde(rename = "SetOp")]
    SetOp(SetOp),
    #[serde(rename = "LockRows")]
    LockRows(LockRows),
    #[serde(rename = "Result")]
    ResultNode(ResultNode),
    #[serde(rename = "Incremental Sort")]
    IncrementalSort(IncrementalSort),
    #[serde(rename = "WindowAgg")]
    WindowAgg(WindowAgg),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Aggregate {
    #[serde(rename = "Parent Relationship")]
    pub parent_relationship: Option<String>,
    #[serde(rename = "Strategy")]
    pub strategy: String,
    #[serde(rename = "Partial Mode")]
    pub partial_mode: String,
    #[serde(rename = "Subplan Name")]
    pub subplan_name: Option<String>,
    #[serde(rename = "Plans")]
    pub children: Option<Vec<PlanNode>>,
    #[serde(rename = "Output")]
    pub output: Option<Vec<String>>,
    #[serde(rename = "Group Key")]
    pub group_keys: Option<Vec<String>>,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WindowAgg {
    #[serde(rename = "Parent Relationship")]
    pub parent_relationship: Option<String>,
    #[serde(rename = "Plans")]
    pub children: Option<Vec<PlanNode>>,
    #[serde(rename = "Output")]
    pub output: Option<Vec<String>>,
    #[serde(rename = "Run Condition")]
    pub run_condition: Option<String>,
}
// scans
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SeqScan {
    #[serde(rename = "Parent Relationship")]
    pub parent_relationship: Option<String>,
    #[serde(rename = "Relation Name")]
    pub relation_name: String,
    #[serde(rename = "Alias")]
    pub alias: Option<String>,
    #[serde(rename = "Filter")]
    pub filter: Option<String>,
    #[serde(rename = "Plan Rows")]
    pub plan_rows: Option<i64>,
    #[serde(rename = "Actual Rows")]
    pub actual_rows: Option<i64>,
    #[serde(rename = "Output")]
    pub output: Option<Vec<String>>,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IndexScan {
    #[serde(rename = "Parent Relationship")]
    pub parent_relationship: Option<String>,
    #[serde(rename = "Relation Name")]
    pub relation_name: String,
    #[serde(rename = "Index Name")]
    pub index_name: String,
    #[serde(rename = "Alias")]
    pub alias: Option<String>,
    #[serde(rename = "Filter")]
    pub filter: Option<String>,
    #[serde(rename = "Scan Direction")]
    pub scan_direction: Option<String>,
    #[serde(rename = "Plan Rows")]
    pub plan_rows: Option<i64>,
    #[serde(rename = "Actual Rows")]
    pub actual_rows: Option<i64>,
    #[serde(rename = "Output")]
    pub output: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IndexOnlyScan {
    #[serde(rename = "Parent Relationship")]
    pub parent_relationship: Option<String>,
    #[serde(rename = "Relation Name")]
    pub relation_name: String,
    #[serde(rename = "Index Name")]
    pub index_name: String,
    #[serde(rename = "Alias")]
    pub alias: Option<String>,
    #[serde(rename = "Filter")]
    pub filter: Option<String>,
    #[serde(rename = "Scan Direction")]
    pub scan_direction: Option<String>,
    #[serde(rename = "Output")]
    pub output: Option<Vec<String>>,
    #[serde(rename = "Plan Rows")]
    pub plan_rows: Option<i64>,
    #[serde(rename = "Actual Rows")]
    pub actual_rows: Option<i64>,
    #[serde(rename = "Index Cond")]
    pub index_cond: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ValueScan {
    #[serde(rename = "Alias")]
    pub alias: Option<String>,
    #[serde(rename = "Filter")]
    pub filter: Option<String>,
    #[serde(rename = "Plan Rows")]
    pub plan_rows: Option<i64>,
    #[serde(rename = "Actual Rows")]
    pub actual_rows: Option<i64>,
    #[serde(rename = "Output")]
    pub output: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubqueryScan {
    #[serde(rename = "Parent Relationship")]
    pub parent_relationship: Option<String>,
    #[serde(rename = "Alias")]
    pub alias: Option<String>,
    #[serde(rename = "Filter")]
    pub filter: Option<String>,
    #[serde(rename = "Output")]
    pub output: Option<Vec<String>>,
    #[serde(rename = "Plan Rows")]
    pub plan_rows: Option<i64>,
    #[serde(rename = "Actual Rows")]
    pub actual_rows: Option<i64>,
    #[serde(rename = "Plans")]
    pub children: Option<Vec<PlanNode>>,
}

// joins
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Hash {
    // this one might be building hashtable
    #[serde(rename = "Parent Relationship")]
    pub parent_relationship: Option<String>,
    #[serde(rename = "Plans")]
    pub children: Option<Vec<PlanNode>>,
    #[serde(rename = "Output")]
    pub output: Option<Vec<String>>,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HashJoin {
    #[serde(rename = "Parent Relationship")]
    pub parent_relationship: Option<String>,
    #[serde(rename = "Join Type")]
    pub join_type: String,
    #[serde(rename = "Inner Unique")]
    pub inner_unique: bool,
    #[serde(rename = "Hash Cond")]
    pub hash_cond: String,
    #[serde(rename = "Plan Rows")]
    pub plan_rows: Option<i64>,
    #[serde(rename = "Actual Rows")]
    pub actual_rows: Option<i64>,
    #[serde(rename = "Join Filter")]
    pub join_filter: Option<String>,
    #[serde(rename = "Actual Loops")]
    pub actual_loops: Option<i64>,
    #[serde(rename = "Plans")]
    pub children: Option<Vec<PlanNode>>,
    #[serde(rename = "Output")]
    pub output: Option<Vec<String>>,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MergeJoin {
    #[serde(rename = "Parent Relationship")]
    pub parent_relationship: Option<String>,
    #[serde(rename = "Join Type")]
    pub join_type: String,
    #[serde(rename = "Inner Unique")]
    pub inner_unique: bool,
    #[serde(rename = "Merge Cond")]
    pub merge_cond: String,
    #[serde(rename = "Plan Rows")]
    pub plan_rows: Option<i64>,
    #[serde(rename = "Actual Rows")]
    pub actual_rows: Option<i64>,
    #[serde(rename = "Actual Loops")]
    pub actual_loops: Option<i64>,
    #[serde(rename = "Join Filter")]
    pub join_filter: Option<String>,
    #[serde(rename = "Plans")]
    pub children: Option<Vec<PlanNode>>,
    #[serde(rename = "Output")]
    pub output: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NestedLoopJoin {
    #[serde(rename = "Parent Relationship")]
    pub parent_relationship: Option<String>,
    #[serde(rename = "Join Type")]
    pub join_type: String,
    #[serde(rename = "Inner Unique")]
    pub inner_unique: bool,
    #[serde(rename = "Plan Rows")]
    pub plan_rows: Option<i64>,
    #[serde(rename = "Actual Rows")]
    pub actual_rows: Option<i64>,
    #[serde(rename = "Join Filter")]
    pub join_filter: Option<String>,
    #[serde(rename = "Plans")]
    pub children: Option<Vec<PlanNode>>,
    #[serde(rename = "Actual Loops")]
    pub actual_loops: Option<i64>,
    #[serde(rename = "Output")]
    pub output: Option<Vec<String>>,
}

// limit, unique, sort
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Limit {
    #[serde(rename = "Parent Relationship")]
    pub parent_relationship: Option<String>,
    #[serde(rename = "Plan Rows")]
    pub limit_rows: i64, // NOTICE: this is actually the statistics, but seemed to be the only way to extract limit row count target
    #[serde(rename = "Plans")]
    pub children: Option<Vec<PlanNode>>,
    #[serde(rename = "Output")]
    pub output: Option<Vec<String>>,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Sort {
    #[serde(rename = "Parent Relationship")]
    pub parent_relationship: Option<String>,
    #[serde(rename = "Sort Key")]
    pub sort_keys: Vec<String>, // will be like [a, b DESC]
    #[serde(rename = "Plans")]
    pub children: Option<Vec<PlanNode>>,
    #[serde(rename = "Output")]
    pub output: Option<Vec<String>>,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IncrementalSort {
    #[serde(rename = "Parent Relationship")]
    pub parent_relationship: Option<String>,
    #[serde(rename = "Sort Key")]
    pub sort_keys: Vec<String>,
    #[serde(rename = "Presorted Key")]
    pub presorted_keys: Vec<String>,
    #[serde(rename = "Output")]
    pub output: Option<Vec<String>>,
    #[serde(rename = "Plans")]
    pub children: Option<Vec<PlanNode>>,
    #[serde(rename = "Run Condition")]
    pub run_condition: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SetOp {
    #[serde(rename = "Strategy")]
    pub strategy: String,
    #[serde(rename = "Plans")]
    pub children: Option<Vec<PlanNode>>,
    #[serde(rename = "Output")]
    pub output: Option<Vec<String>>,
    #[serde(rename = "Command")]
    pub command: Option<String>,
}

impl PlanNode {
    pub fn get_children(&self) -> Option<Vec<PlanNode>> {
        match self {
            PlanNode::Aggregate(aggregate) => aggregate.children.clone(),
            PlanNode::SeqScan(_) => None,
            PlanNode::IndexScan(_) => None,
            PlanNode::Hash(hash) => hash.children.clone(),
            PlanNode::HashJoin(hash_join) => hash_join.children.clone(),
            PlanNode::MergeJoin(merge_join) => merge_join.children.clone(),
            PlanNode::Limit(limit) => limit.children.clone(),
            PlanNode::Sort(sort) => sort.children.clone(),
            PlanNode::Unique(unique) => unique.children.clone(),
            PlanNode::Append(append) => append.children.clone(),
            PlanNode::Gather(gather) => gather.children.clone(),
            PlanNode::GatherMerge(gather_merge) => gather_merge.children.clone(),
            PlanNode::NestedLoopJoin(nested_loop_join) => nested_loop_join.children.clone(),
            PlanNode::IndexOnlyScan(_) => None,
            PlanNode::Memoize(memoize) => memoize.children.clone(),
            PlanNode::ValueScan(_) => None,
            PlanNode::SubqueryScan(subquery_scan) => subquery_scan.children.clone(),
            PlanNode::SetOp(set_op) => set_op.children.clone(),
            PlanNode::LockRows(lock_rows) => lock_rows.children.clone(),
            PlanNode::ResultNode(_) => None,
            PlanNode::IncrementalSort(incremental_sort) => incremental_sort.children.clone(),
            PlanNode::WindowAgg(window_agg) => window_agg.children.clone(),
        }
    }
}

// Group of all scan operators
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ScanNode {
    SeqScan(SeqScan),
    IndexScan(IndexScan),
    IndexOnlyScan(IndexOnlyScan),
    ValueScan(ValueScan),
    SubqueryScan(SubqueryScan),
}

impl ScanNode {
    pub fn get_alias(&self) -> Option<String> {
        match self {
            ScanNode::SeqScan(seq_scan) => seq_scan.alias.clone(),
            ScanNode::IndexScan(index_scan) => index_scan.alias.clone(),
            ScanNode::IndexOnlyScan(index_only_scan) => index_only_scan.alias.clone(),
            ScanNode::ValueScan(value_scan) => value_scan.alias.clone(),
            ScanNode::SubqueryScan(subquery_scan) => subquery_scan.alias.clone(),
        }
    }

    pub fn get_filter(&self) -> Option<String> {
        match self {
            ScanNode::SeqScan(seq_scan) => seq_scan.filter.clone(),
            ScanNode::IndexScan(index_scan) => index_scan.filter.clone(),
            ScanNode::IndexOnlyScan(index_only_scan) => index_only_scan.filter.clone(),
            ScanNode::ValueScan(value_scan) => value_scan.filter.clone(),
            ScanNode::SubqueryScan(subquery_scan) => subquery_scan.filter.clone(),
        }
    }

    pub fn get_relation_name(&self) -> String {
        match self {
            ScanNode::SeqScan(seq_scan) => seq_scan.relation_name.clone(),
            ScanNode::IndexScan(index_scan) => index_scan.relation_name.clone(),
            ScanNode::IndexOnlyScan(index_only_scan) => index_only_scan.relation_name.clone(),
            // ValueScan and SubqueryScan Node does not have a relation_name, use alias as a replacement
            ScanNode::ValueScan(value_scan) => value_scan.alias.clone().unwrap_or_default(),
            ScanNode::SubqueryScan(subquery_scan) => {
                subquery_scan.alias.clone().unwrap_or_default()
            }
        }
    }

    pub fn get_index_name(&self) -> Option<String> {
        match self {
            ScanNode::SeqScan(_) => None,
            ScanNode::IndexScan(index_scan) => Some(index_scan.index_name.clone()),
            ScanNode::IndexOnlyScan(index_only_scan) => Some(index_only_scan.index_name.clone()),
            ScanNode::ValueScan(_) => None,
            ScanNode::SubqueryScan(_) => None,
        }
    }

    pub fn get_output(&self) -> Option<Vec<String>> {
        match self {
            ScanNode::SeqScan(seq_scan) => seq_scan.output.clone(),
            ScanNode::IndexScan(index_scan) => index_scan.output.clone(),
            ScanNode::IndexOnlyScan(index_only_scan) => index_only_scan.output.clone(),
            ScanNode::ValueScan(value_scan) => value_scan.output.clone(),
            ScanNode::SubqueryScan(subquery_scan) => subquery_scan.output.clone(),
        }
    }

    pub fn get_card(&self) -> Option<i64> {
        match self {
            ScanNode::SeqScan(seq_scan) => seq_scan.actual_rows,
            ScanNode::IndexScan(index_scan) => index_scan.actual_rows,
            ScanNode::IndexOnlyScan(index_only_scan) => index_only_scan.actual_rows,
            ScanNode::ValueScan(value_scan) => value_scan.actual_rows,
            ScanNode::SubqueryScan(subquery_scan) => subquery_scan.actual_rows,
        }
    }
}

// Group of all join operators
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum JoinNode {
    HashJoin(HashJoin),
    MergeJoin(MergeJoin),
    NestedLoopJoin(NestedLoopJoin),
}

impl JoinNode {
    pub fn get_children(&self) -> Option<Vec<PlanNode>> {
        match self {
            JoinNode::HashJoin(hash_join) => hash_join.children.clone(),
            JoinNode::MergeJoin(merge_join) => merge_join.children.clone(),
            JoinNode::NestedLoopJoin(nested_loop_join) => nested_loop_join.children.clone(),
        }
    }

    pub fn get_left(&self) -> Option<PlanNode> {
        match self {
            JoinNode::HashJoin(hash_join) => hash_join
                .children
                .as_ref()
                .and_then(|children| children.get(0).cloned()),
            JoinNode::MergeJoin(merge_join) => merge_join
                .children
                .as_ref()
                .and_then(|children| children.get(0).cloned()),
            JoinNode::NestedLoopJoin(nested_loop_join) => nested_loop_join
                .children
                .as_ref()
                .and_then(|children| children.get(0).cloned()),
        }
    }

    pub fn get_right(&self) -> Option<PlanNode> {
        match self {
            JoinNode::HashJoin(hash_join) => hash_join
                .children
                .as_ref()
                .and_then(|children| children.get(1).cloned()),
            JoinNode::MergeJoin(merge_join) => merge_join
                .children
                .as_ref()
                .and_then(|children| children.get(1).cloned()),
            JoinNode::NestedLoopJoin(nested_loop_join) => nested_loop_join
                .children
                .as_ref()
                .and_then(|children| children.get(1).cloned()),
        }
    }

    pub fn get_join_type(&self) -> String {
        match self {
            JoinNode::HashJoin(hash_join) => hash_join.join_type.clone(),
            JoinNode::MergeJoin(merge_join) => merge_join.join_type.clone(),
            JoinNode::NestedLoopJoin(nested_loop_join) => nested_loop_join.join_type.clone(),
        }
    }

    pub fn get_join_filter(&self) -> Option<String> {
        match self {
            JoinNode::HashJoin(hash_join) => hash_join.join_filter.clone(),
            JoinNode::MergeJoin(merge_join) => merge_join.join_filter.clone(),
            JoinNode::NestedLoopJoin(_) => None,
        }
    }

    pub fn get_inner_unique(&self) -> bool {
        match self {
            JoinNode::HashJoin(hash_join) => hash_join.inner_unique,
            JoinNode::MergeJoin(merge_join) => merge_join.inner_unique,
            JoinNode::NestedLoopJoin(nested_loop_join) => nested_loop_join.inner_unique,
        }
    }

    pub fn get_condition(&self) -> Option<String> {
        match self {
            JoinNode::HashJoin(hash_join) => Some(hash_join.hash_cond.clone()),
            JoinNode::MergeJoin(merge_join) => Some(merge_join.merge_cond.clone()),
            JoinNode::NestedLoopJoin(_) => None,
        }
    }

    pub fn get_output(&self) -> Option<Vec<String>> {
        match self {
            JoinNode::HashJoin(hash_join) => hash_join.output.clone(),
            JoinNode::MergeJoin(merge_join) => merge_join.output.clone(),
            JoinNode::NestedLoopJoin(nested_loop_join) => nested_loop_join.output.clone(),
        }
    }

    pub fn get_card(&self) -> Option<i64> {
        match self {
            JoinNode::HashJoin(hash_join) => {
                match (hash_join.actual_rows, hash_join.actual_loops) {
                    (Some(rows), Some(loops)) => Some(rows * loops),
                    (Some(rows), None) => Some(rows),
                    _ => None,
                }
            },
            JoinNode::MergeJoin(merge_join) => {
                match (merge_join.actual_rows, merge_join.actual_loops) {
                    (Some(rows), Some(loops)) => Some(rows * loops),
                    (Some(rows), None) => Some(rows),
                    _ => None,
                }
            },
            JoinNode::NestedLoopJoin(nested_loop_join) => {
                match (nested_loop_join.actual_rows, nested_loop_join.actual_loops) {
                    (Some(rows), Some(loops)) => Some(rows * loops),
                    (Some(rows), None) => Some(rows),
                    _ => None,
                }
            },
        }
    }
}

// Group of all set operators
#[derive(Debug, Serialize, Deserialize, Clone)]

pub enum SetNode {
    Append(Append),
    SetOp(SetOp),
}

impl SetNode {
    pub fn get_children(&self) -> Option<Vec<PlanNode>> {
        match self {
            SetNode::Append(append) => append.children.clone(),
            SetNode::SetOp(set_op) => set_op.children.clone(),
        }
    }

    pub fn set_children(&mut self, children: Vec<PlanNode>) {
        match self {
            SetNode::Append(append) => append.children = Some(children),
            SetNode::SetOp(set_op) => set_op.children = Some(children),
        }
    }

    pub fn get_operator(&self) -> SetOperator {
        match self {
            SetNode::Append(_) => SetOperator::Union,
            SetNode::SetOp(set_op) => match set_op.command.as_deref() {
                Some("Intersect") | Some("Intersect All") => SetOperator::Intersect,
                Some("Except") | Some("Except All") => SetOperator::Except,
                Some(command) => panic!("Unsupported SetOperator: {}", command),
                _ => panic!("Missing command in SetOperator."),
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]

pub struct Unique {
    #[serde(rename = "Parent Relationship")]
    pub parent_relationship: Option<String>,
    #[serde(rename = "Plans")]
    pub children: Option<Vec<PlanNode>>,
    #[serde(rename = "Output")]
    pub output: Option<Vec<String>>,
}
// "useless" nodes
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Append {
    #[serde(rename = "Parent Relationship")]
    pub parent_relationship: Option<String>,
    #[serde(rename = "Subplans Removed")]
    pub subplans_removed: i64,
    #[serde(rename = "Plans")]
    pub children: Option<Vec<PlanNode>>,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Gather {
    #[serde(rename = "Parent Relationship")]
    pub parent_relationship: Option<String>,
    #[serde(rename = "Plans")]
    pub children: Option<Vec<PlanNode>>,
    #[serde(rename = "Workers Planned")]
    pub num_workers: Option<i64>,
    #[serde(rename = "Output")]
    pub output: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]

pub struct GatherMerge {
    #[serde(rename = "Parent Relationship")]
    pub parent_relationship: Option<String>,
    #[serde(rename = "Plans")]
    pub children: Option<Vec<PlanNode>>,
    #[serde(rename = "Output")]
    pub output: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]

pub struct Memoize {
    #[serde(rename = "Parent Relationship")]
    pub parent_relationship: Option<String>,
    #[serde(rename = "Plans")]
    pub children: Option<Vec<PlanNode>>,
    #[serde(rename = "Output")]
    pub output: Option<Vec<String>>,
    #[serde(rename = "Cache Key")]
    pub cache_key: Option<String>,
    #[serde(rename = "Cache Mode")]
    pub cache_mode: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LockRows {
    #[serde(rename = "Parent Relationship")]
    pub parent_relationship: Option<String>,
    #[serde(rename = "Plans")]
    pub children: Option<Vec<PlanNode>>,
    #[serde(rename = "Output")]
    pub output: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResultNode {
    #[serde(rename = "Parent Relationship")]
    pub parent_relationship: Option<String>,
    #[serde(rename = "Subplan Name")]
    pub subplan_name: Option<String>,
    #[serde(rename = "Output")]
    pub output: Option<Vec<String>>,
    #[serde(rename = "One-Time Filter")]
    pub filter: Option<String>,
}

impl ResultNode {
    pub fn get_filter(&self) -> Option<&String> {
        self.filter.as_ref()
    }
}

/// convert postgres plan json string to a tree of PlanNode's
/// # Arguments
/// + `input_json_path` - input path to the json file
/// # Returns
/// result type of PlanNode tree or an `serde` parsing error
pub fn postgres2plan(input_json: &str) -> Result<PlanNode, serde_json::Error> {
    parse_json(input_json)
}

pub fn postgres2planroot(input_json: &str) -> Result<PlanRoot, serde_json::Error> {
    parse_json_to_root(input_json)
}

/// parse the input json into a struct representing postgres plan tree
fn parse_json(input_json: &str) -> Result<PlanNode, serde_json::Error> {
    let plan_roots: Vec<PlanRoot> = serde_json::from_str(input_json)?;
    let plan = plan_roots.first().unwrap().plan.clone();
    Ok(plan)
}

/// parse the input json into a struct representing postgres plan tree
fn parse_json_to_root(input_json: &str) -> Result<PlanRoot, serde_json::Error> {
    let plan_roots: Vec<PlanRoot> = serde_json::from_str(input_json)?;
    Ok(plan_roots.first().unwrap().clone())
}

#[cfg(test)]
mod test_parse_json {
    use super::*;
    use test_each_file::test_each_path;
    fn test_input_plan(input_path: &std::path::Path) {
        let input = std::fs::read_to_string(input_path).expect("Failed to read input file");
        let result = postgres2plan(&input).unwrap();
        println!("{:#?}", result)
    }

    // Runs tests for each plan in resources/test_json. Each json also needs to have a corresponding sql file in resources/test_sql.
    test_each_path! {
        in "resources/test_json"  => test_input_plan
    }
}
