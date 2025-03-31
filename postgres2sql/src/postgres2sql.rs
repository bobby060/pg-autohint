/// External-facing API for postgres2sql
use crate::{plan2ast::Visit, postgresplan};
/// Provides api for converting JSON string (and json path) to SQL

/// Steps:
/// 1. Deserialize the JSON string into a Postgres plan
/// 2. Convert the Postgres plan to a datafusion AST
/// 3. Add hints
pub fn postgres2sql(json: String) -> Result<String, String> {
    let plan = postgresplan::postgres2plan(&json).map_err(|e| e.to_string())?;

    let ast = plan.visit_plan_node()?;

    let sql = ast.to_string();

    Ok(sql)
}

// #[cfg(test)]
// mod test {
//     use super::*;
//     use crate::test_utils::*;
//     use std::path::Path;
//     use test_each_file::test_each_path;

//     // #[test]
//     fn test_input_plan(input_path: &std::path::Path) {
//         let input = std::fs::read_to_string(input_path).expect("Failed to read input file");
//         let new_sql = postgres2sql(input).expect("Failed to parse input");

//         let sql_path = input_path.to_str().unwrap().replace("json", "sql");
//         let original_sql = std::fs::read_to_string(Path::new(sql_path.as_str()))
//             .expect("Failed to read original sql file");

//         println!("Original SQL: {}", &original_sql);

//         println!("New SQL: {}", &new_sql);

//         correctness_test(
//             "imdb",
//             &original_sql,
//             &new_sql,
//             new_sql.contains("ORDER BY"),
//         );
//     }

//     // Runs tests for each plan in resources/test_json. Each json also needs to have a corresponding sql file in resources/test_sql.
//     test_each_path! {
//         in "resources/test_json"  => test_input_plan
//     }
// }