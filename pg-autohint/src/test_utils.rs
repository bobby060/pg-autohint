use crate::connector::establish_connection;
use crate::optimize::Optimizer;
use crate::rule::Rule;
use postgres::types::Type;

/// Test correctness of the new sql query by comparing the result with the original sql query
///
/// Current limitations: Only works for ordered queries. Only works for types in the IMDB dataset
///
/// # Arguments
///
/// * `db_name`: The name of the database to connect to (should be running locally)
/// * `original_sql`: The original sql query
/// * `new_sql`: The new sql query
pub fn correctness_test(db_name: &str, original_sql: &str, new_sql: &str, ordered: bool) {
    let mut conn = establish_connection(db_name, "postgres", "postgres", "localhost", "5432");

    let result = conn.query(new_sql, &[]).unwrap();
    let expected = conn.query(original_sql, &[]).unwrap();

    let mut expected_rows: Vec<Vec<String>> = vec![];
    let mut actual_rows: Vec<Vec<String>> = vec![];

    for (result_row, expected_row) in result.iter().zip(expected.iter()) {
        let mut actual_row_str = vec![];
        let mut expected_row_str = vec![];
        for i in 0..result_row.len() {
            let t: &Type = result_row.columns()[i].type_();

            // All types in our test dataset. Would need to add more to support all postgres types.
            match t {
                &Type::INT2 => {
                    let value1: Option<i16> = result_row.try_get(i).ok();
                    let value2: Option<i16> = expected_row.try_get(i).ok();
                    actual_row_str.push(value1.unwrap_or_else(|| 0).to_string());
                    expected_row_str.push(value2.unwrap_or_else(|| 0).to_string());
                }
                &Type::INT4 => {
                    let value1: Option<i32> = result_row.try_get(i).ok();
                    let value2: Option<i32> = expected_row.try_get(i).ok();
                    actual_row_str.push(value1.unwrap_or_else(|| 0).to_string());
                    expected_row_str.push(value2.unwrap_or_else(|| 0).to_string());
                }
                &Type::INT8 => {
                    let value1: Option<i64> = result_row.try_get(i).ok();
                    let value2: Option<i64> = expected_row.try_get(i).ok();
                    actual_row_str.push(value1.unwrap_or_else(|| 0).to_string());
                    expected_row_str.push(value2.unwrap_or_else(|| 0).to_string());
                }
                &Type::TEXT => {
                    let value1: Option<&str> = result_row.try_get(i).ok();
                    let value2: Option<&str> = expected_row.try_get(i).ok();
                    actual_row_str.push(value1.unwrap_or_else(|| "NULL").to_string());
                    expected_row_str.push(value2.unwrap_or_else(|| "NULL").to_string());
                }
                &Type::VARCHAR => {
                    let value1: Option<&str> = result_row.try_get(i).ok();
                    let value2: Option<&str> = expected_row.try_get(i).ok();
                    actual_row_str.push(value1.unwrap_or_else(|| "NULL").to_string());
                    expected_row_str.push(value2.unwrap_or_else(|| "NULL").to_string());
                }
                &Type::BOOL => {
                    let value1: Option<bool> = result_row.try_get(i).ok();
                    let value2: Option<bool> = expected_row.try_get(i).ok();
                    actual_row_str.push(value1.unwrap_or_else(|| false).to_string());
                    expected_row_str.push(value2.unwrap_or_else(|| false).to_string());
                }
                &Type::NUMERIC => {
                    let value1: Option<f64> = result_row.try_get(i).ok();
                    let value2: Option<f64> = expected_row.try_get(i).ok();
                    actual_row_str.push(value1.unwrap_or_else(|| 0.0).to_string());
                    expected_row_str.push(value2.unwrap_or_else(|| 0.0).to_string());
                }
                _ => {
                    panic!("Unsupported type: {:?}", t);
                }
            }
        }

        actual_rows.push(actual_row_str);
        expected_rows.push(expected_row_str);

        // Prevent overloading tester
        if actual_rows.len() > 100 {
            break;
        }
    }

    if ordered {
        assert_eq!(actual_rows, expected_rows);
    } else {
        assert_eq!(actual_rows.sort(), expected_rows.sort());
    }
}

pub fn rule_test(rule: Box<dyn Rule>, sql: &str, is_analyze: bool) {
    let mut conn = establish_connection("imdb", "postgres", "postgres", "localhost", "5432");

    let mut optimizer = Optimizer { rules: vec![rule] };

    let new_sql = optimizer.optimize(sql, &mut conn, is_analyze);

    correctness_test(
        "imdb",
        sql,
        new_sql.unwrap().as_str(),
        sql.contains("ORDER BY"),
    );
}
