pub mod connector;
pub mod hints;
pub mod optimize;
pub mod plan2ast;
pub mod postgres2sql;
pub mod postgresplan;
pub mod rule;
pub mod rules;

#[cfg(test)]
pub mod test_utils;

use connector::*;

pub fn convert_sql_file_to_plan(sql_file: &str, json_out_path: &str) {
    let sql = std::fs::read_to_string(sql_file).expect("Failed to read sql file");

    let mut client = establish_connection("postgres", "postgres", "postgres", "localhost", "5432");

    let plan = query_to_plan(&sql, &mut client);

    let json_out = std::fs::File::create(json_out_path).expect("Failed to create json out file");

    serde_json::to_writer(json_out, &plan).expect("Failed to write json out file");
}
