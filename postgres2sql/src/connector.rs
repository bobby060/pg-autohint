use crate::postgres2plan::PlanWrapper;
use postgres::types::Json;
use postgres::{Client, NoTls};

/// Establish a connection to a postgres database
///
/// # Arguments
///
/// * `db_name`: The name of the database to connect to
/// * `user`: The username to connect to the database
/// * `password`: The password to connect to the database
/// * `host`: The host to connect to the database
/// * `port`: The port to connect to the database
///
/// # Returns postgres::Client
pub fn establish_connection(
    db_name: &str,
    user: &str,
    password: &str,
    host: &str,
    port: &str,
) -> Client {
    let database_url = format!(
        "postgres://{}:{}@{}:{}/{}",
        user, password, host, port, db_name
    );
    Client::connect(&database_url, NoTls).expect(&format!("Error connecting to {}", database_url))
}

/// Convert a SQL query to a PlanWrapper struct
///
///     Unwrap the PlanWrapper to get the root PlanNode
///
/// # Arguments
///
/// * `query`: The query to convert to a SQL query to serialized postgres plan
///
/// # Returns Vec<PlanWrapper>
pub fn query_to_plan(query: &str, conn: &mut Client) -> Vec<PlanWrapper> {
    let prefix = "EXPLAIN (FORMAT JSON, VERBOSE TRUE) ";
    let result = conn.query(&(prefix.to_string() + query), &[]).unwrap();

    let value: Option<Json<Vec<PlanWrapper>>> = result.get(0).unwrap().get(0);

    value.unwrap().0
}

#[cfg(test)]
mod test_connector {
    use super::*;

    #[test]
    fn test_establish_connection() {
        let _conn = establish_connection("imdb", "postgres", "postgres", "localhost", "5432");
    }

    #[test]
    fn query_to_plan_test() {
        let sql = " SELECT * FROM name_basics limit 10";

        let mut conn = establish_connection("imdb", "postgres", "postgres", "localhost", "5432");

        let result = query_to_plan(sql, &mut conn);

        println!("{:?}", result);
    }
}
