/// Represents a Postgres plan
/// Deserializes a Postgres plan from a JSON string
use serde::*;

/// Wrapper for parsing the whole plan json 
#[derive(Debug, Serialize, Deserialize)]
struct PlanWrapper {
    #[serde(rename = "Plan")]
    plan: PlanNode
}

/// Plan node type enum for `serde` json parsing
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "Node Type")]
pub enum PlanNode {
    Aggregate {
        #[serde(rename = "Strategy")]
        strategy: String,
        #[serde(rename = "Partial Mode")]
        partial_mode: String,
        #[serde(rename = "Plans")]
        children: Option<Vec<PlanNode>>,
    },
    Gather {
        #[serde(rename = "Plans")]
        children: Option<Vec<PlanNode>>,
    },
    #[serde(rename = "Seq Scan")]
    SeqScan {
        #[serde(rename = "Parent Relationship")]
        parent_relationship: String,
        #[serde(rename = "Relation Name")]
        relation_name: String,
        #[serde(rename = "Alias")]
        alias: String,
        #[serde(rename = "Filter")]
        filter: Option<String>,
    },
    #[serde(rename = "Hash Join")]
    HashJoin {
        #[serde(rename = "Parent Relationship")]
        parent_relationship: String,
        #[serde(rename = "Join Type")]
        join_type: String,
        #[serde(rename = "Inner Unique")]
        inner_unique: bool,
        #[serde(rename = "Hash Cond")]
        hash_cond: String,
        #[serde(rename = "Plans")]
        children: Option<Vec<PlanNode>>
    },
    #[serde(rename = "Hash")]
    Hash {
        #[serde(rename = "Parent Relationship")]
        parent_relationship: String,
        #[serde(rename = "Plans")]
        children: Option<Vec<PlanNode>>
    }
}

/// convert postgres plan json string to a tree of PlanNode's
/// # Arguments
/// + `input_json_path` - input path to the json file
/// # Returns
/// result type of PlanNode tree or an `serde` parsing error 
pub fn postgres2plan(input_json_path: &str) -> Result<PlanNode, serde_json::Error> {
    parse_json(input_json_path)
}

/// parse the input json into a struct representing postgres plan tree
fn parse_json(input_json_path: &str) -> Result<PlanNode, serde_json::Error> {
  let plan_wrappers: Vec<PlanWrapper> = serde_json::from_str(input_json_path)?;
  let plan = plan_wrappers.first().unwrap().plan.clone();
  Ok(plan)
}

#[cfg(test)]
mod test_parse_json {
    use super::*;

    #[test]
    fn test_parse_json () {
    let input_path = "test_jsons/q1.json";
    let input = std::fs::read_to_string(input_path).expect("Failed to read input file");
    println!("{:#?}", parse_json(&input).unwrap());
    }

    #[test]
    fn test_parse_json_2 () {
    let input_path = "test_jsons/q3.json";
    let input = std::fs::read_to_string(input_path).expect("Failed to read input file");
    println!("{:#?}", parse_json(&input).unwrap());
    }
}