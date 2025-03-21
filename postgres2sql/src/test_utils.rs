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
pub fn correctness_test(db_name: &str, original_sql: &str, new_sql: &str) {
    let mut conn = establish_connection(db_name);

    let result = conn.query(new_sql, &[]).unwrap();
    let expected = conn.query(original_sql, &[]).unwrap();

    for (result_row, expected_row) in result.iter().zip(expected.iter()) {
        for i in 0..result_row.len() {
            let t: &Type = result_row.columns()[i].type_();

            // TODO: Currently only works for queries that are ordered

            // All types in our test dataset. Would need to add more to support all postgres types.
            match t {
                &Type::INT2 => {
                    let value1: Option<i16> = result_row.try_get(i).ok();
                    let value2: Option<i16> = expected_row.try_get(i).ok();
                    assert_eq!(value1, value2);
                }
                &Type::INT4 => {
                    let value1: Option<i32> = result_row.try_get(i).ok();
                    let value2: Option<i32> = expected_row.try_get(i).ok();
                    assert_eq!(value1, value2);
                }
                &Type::INT8 => {
                    let value1: Option<i64> = result_row.try_get(i).ok();
                    let value2: Option<i64> = expected_row.try_get(i).ok();
                    assert_eq!(value1, value2);
                }
                &Type::TEXT => {
                    let value1: Option<&str> = result_row.try_get(i).ok();
                    let value2: Option<&str> = expected_row.try_get(i).ok();
                    assert_eq!(value1, value2);
                }
                &Type::VARCHAR => {
                    let value1: Option<&str> = result_row.try_get(i).ok();
                    let value2: Option<&str> = expected_row.try_get(i).ok();
                    assert_eq!(value1, value2);
                }
                &Type::BOOL => {
                    let value1: Option<bool> = result_row.try_get(i).ok();
                    let value2: Option<bool> = expected_row.try_get(i).ok();
                    assert_eq!(value1, value2);
                }
                _ => {
                    panic!("Unsupported type: {:?}", t);
                }
            }
        }
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
    );
}
