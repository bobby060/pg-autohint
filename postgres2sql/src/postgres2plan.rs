/// Represents a Postgres plan

/// Deserializes a Postgres plan from a JSON string
use serde::*;

/// Wrapper
#[derive(Debug, Serialize, Deserialize)]
struct PlanWrapper {
    #[serde(rename = "Plan")]
    plan: PlanNode
}

/// Plan node types
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


pub fn postgres2plan() {
}

/// parse the input json into a struct representing postgres plan tree
pub fn parse_json(input: &str) -> Result<PlanNode, serde_json::Error> {
  let plan_wrappers: Vec<PlanWrapper> = serde_json::from_str(input)?;
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