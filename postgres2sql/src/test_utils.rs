use postgres::{types::Type, Client, NoTls};
fn establish_connection(db_name: &str) -> Client {
    let database_url = format!("postgres://postgres:postgres@localhost:5432/{}", db_name);
    Client::connect(&database_url, NoTls).expect(&format!("Error connecting to {}", database_url))
}

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
    let mut conn = establish_connection(db_name);

    let result = conn.query(new_sql, &[]).unwrap();
    let expected = conn.query(original_sql, &[]).unwrap();

    let mut expected_rows: Vec<Vec<String>> = vec![];
    let mut actual_rows: Vec<Vec<String>> = vec![];

    for (result_row, expected_row) in result.iter().zip(expected.iter()) {
        let mut actual_row_str = vec![];
        let mut expected_row_str = vec![];
        for i in 0..result_row.len() {
            let t: &Type = result_row.columns()[i].type_();

            // TODO: Currently only works for queries that are ordered

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
                _ => {
                    panic!("Unsupported type: {:?}", t);
                }
            }
        }

        actual_rows.push(actual_row_str);
        expected_rows.push(expected_row_str);
    }

    if ordered {
        assert_eq!(actual_rows, expected_rows);
    } else {
        assert_eq!(actual_rows.sort(), expected_rows.sort());
    }
}

#[test]
fn test_establish_connection() {
    let _conn = establish_connection("imdb");
}

#[test]
fn test_correctness_test() {
    correctness_test(
        "imdb",
        "SELECT * FROM name_basics limit 10",
        "SELECT * FROM name_basics limit 10",
        false,
    );
}
