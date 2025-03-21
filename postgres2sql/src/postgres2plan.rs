/// Represents a Postgres plan
/// Deserializes a Postgres plan from a JSON string
use serde::*;

/// Wrapper for parsing the whole plan json
#[derive(Debug, Serialize, Deserialize)]
struct PlanWrapper {
    #[serde(rename = "Plan")]
    plan: PlanNode,
}

/// Plan node type enum for `serde` json parsing
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "Node Type")]
pub enum PlanNode {
    Aggregate {
        #[serde(rename = "Parent Relationship")]
        parent_relationship: Option<String>,
        #[serde(rename = "Strategy")]
        strategy: String,
        #[serde(rename = "Partial Mode")]
        partial_mode: String,
        #[serde(rename = "Subplan Name")]
        subplan_name: Option<String>,
        #[serde(rename = "Plans")]
        children: Option<Vec<PlanNode>>,
    },
    // scans
    #[serde(rename = "Seq Scan")]
    SeqScan {
        #[serde(rename = "Parent Relationship")]
        parent_relationship: Option<String>,
        #[serde(rename = "Relation Name")]
        relation_name: String,
        #[serde(rename = "Alias")]
        alias: Option<String>,
        #[serde(rename = "Filter")]
        filter: Option<String>,
    },
    #[serde(rename = "Index Scan")]
    IndexScan {
        #[serde(rename = "Parent Relationship")]
        parent_relationship: Option<String>,
        #[serde(rename = "Relation Name")]
        relation_name: String,
        #[serde(rename = "Index Name")]
        index_name: String,
        #[serde(rename = "Alias")]
        alias: Option<String>,
        #[serde(rename = "Filter")]
        filter: Option<String>,
        #[serde(rename = "Scan Direction")]
        scan_direction: Option<String>,
    },
    // joins
    Hash {     // this one might be building hashtable
        #[serde(rename = "Parent Relationship")]
        parent_relationship: Option<String>,
        #[serde(rename = "Plans")]
        children: Option<Vec<PlanNode>>,
    },
    #[serde(rename = "Hash Join")]
    HashJoin {
        #[serde(rename = "Parent Relationship")]
        parent_relationship: Option<String>,
        #[serde(rename = "Join Type")]
        join_type: String,
        #[serde(rename = "Inner Unique")]
        inner_unique: bool,
        #[serde(rename = "Hash Cond")]
        hash_cond: String,
        #[serde(rename = "Join Filter")]
        join_filter: Option<String>,
        #[serde(rename = "Plans")]
        children: Option<Vec<PlanNode>>,
    },
    #[serde(rename = "Merge Join")]
    MergeJoin {
        #[serde(rename = "Parent Relationship")]
        parent_relationship: Option<String>,
        #[serde(rename = "Join Type")]
        join_type: String,
        #[serde(rename = "Inner Unique")]
        inner_unique: bool,
        #[serde(rename = "Merge Cond")]
        merge_cond: String,
        #[serde(rename = "Join Filter")]
        join_filter: Option<String>,
        #[serde(rename = "Plans")]
        children: Option<Vec<PlanNode>>,
    },
    // limit, unique, sort
    Limit {
        #[serde(rename = "Parent Relationship")]
        parent_relationship: Option<String>,
        #[serde(rename = "Plan Rows")]
        limit_rows: i64, // NOTICE: this is actually the statistics, but seemed to be the only way to extract limit row count target
        #[serde(rename = "Plans")]
        children: Option<Vec<PlanNode>>,
    },
    Sort {
        #[serde(rename = "Parent Relationship")]
        parent_relationship: Option<String>,
        #[serde(rename = "Sort Key")]
        sort_keys: Vec<String>, // will be like [a, b DESC]
        #[serde(rename = "Plans")]
        children: Option<Vec<PlanNode>>,
    },
    Unique {
        #[serde(rename = "Parent Relationship")]
        parent_relationship: Option<String>,
        #[serde(rename = "Plans")]
        children: Option<Vec<PlanNode>>,
    },
    // "useless" nodes
    Append {
        #[serde(rename = "Parent Relationship")]
        parent_relationship: Option<String>,
        #[serde(rename = "Subplans Removed")]
        subplans_removed: i64,
        #[serde(rename = "Plans")]
        children: Option<Vec<PlanNode>>,
    },
    Gather {
        #[serde(rename = "Parent Relationship")]
        parent_relationship: Option<String>,
        #[serde(rename = "Plans")]
        children: Option<Vec<PlanNode>>,
    },
}

/// convert postgres plan json string to a tree of PlanNode's
/// # Arguments
/// + `input_json_path` - input path to the json file
/// # Returns
/// result type of PlanNode tree or an `serde` parsing error
pub fn postgres2plan(input_json: &str) -> Result<PlanNode, serde_json::Error> {
    parse_json(input_json)
}

/// parse the input json into a struct representing postgres plan tree
fn parse_json(input_json: &str) -> Result<PlanNode, serde_json::Error> {
    let plan_wrappers: Vec<PlanWrapper> = serde_json::from_str(input_json)?;
    let plan = plan_wrappers.first().unwrap().plan.clone();
    Ok(plan)
}

#[cfg(test)]
mod test_parse_json {
    use super::*;
    use test_each_file::test_each_path;
    fn test_input_plan(input_path: &std::path::Path) {
        let input = std::fs::read_to_string(input_path).expect("Failed to read input file");
        postgres2plan(&input).unwrap();
    }

    // Runs tests for each plan in resources/test_json. Each json also needs to have a corresponding sql file in resources/test_sql.
    test_each_path! {
        in "resources/test_json"  => test_input_plan
    }
}
