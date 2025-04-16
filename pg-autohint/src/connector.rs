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

#[cfg(test)]
mod test_connector {
    use super::*;

    #[test]
    fn test_establish_connection() {
        let _conn = establish_connection("imdb", "postgres", "postgres", "localhost", "5432");
    }
}
