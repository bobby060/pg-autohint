use crate::model::{hints::PgHint, hints::PgHintList, postgresplan::PlanRoot};
use postgres::types::Json;
use postgres::{Client, Row};
use std::fmt;
use std::fmt::Display;

/// Represents a sql query with different relevant pieces of internal metadata
///
/// to_string() -> always a valid SQL query
///
/// Timeout feature only works when query is executed via execute() or get_plan().
///
/// # Fields
///
/// * `sql`: The sql query
/// * `hints`: The hints for the query
/// * `timeout`: The timeout for the query in milliseconds
pub struct Query {
    sql: String,
    hints: Option<PgHintList>,
    timeout: Option<TimeOut>,
}

impl Query {
    /// Create a new Query
    ///
    /// # Arguments
    ///
    /// * `sql`: The sql query
    /// * `hints`: List of hints to be added to the query
    /// * `timeout`: The timeout for the query
    pub fn new(sql: String, hints: Option<PgHintList>, timeout: Option<TimeOut>) -> Self {
        Query {
            sql: sql,
            hints: hints,
            timeout: timeout,
        }
    }

    /// Set the timeout for the query
    ///
    /// # Arguments
    ///
    /// * `timeout`: The timeout for the query
    pub fn set_timeout(&mut self, timeout: TimeOut) {
        self.timeout = Some(timeout);
    }

    /// Add a hint to the query
    pub fn add_hint(&mut self, hint: PgHint) {
        if self.hints.is_none() {
            self.hints = Some(PgHintList::new());
        }
        self.hints.as_mut().unwrap().add_hint(hint);
    }

    /// Add a list of hints to the query
    ///
    /// # Arguments
    ///
    /// * `hint_list`: The list of hints to add
    pub fn add_hint_list(&mut self, hint_list: PgHintList) {
        if self.hints.is_none() {
            self.hints = Some(PgHintList::new());
        }
        self.hints.as_mut().unwrap().concat_hint_list(hint_list);
    }

    pub fn get_original_sql(&self) -> &str {
        &self.sql
    }

    /// Alias to express Query as a valid sql statement
    ///
    /// # Returns
    ///
    /// A string representing the query
    pub fn to_sql(&self) -> String {
        self.to_string()
    }

    /// Get the hints for the query
    ///
    /// # Returns
    ///
    /// A reference to the hints for the query
    pub fn get_hints(&self) -> Option<&PgHintList> {
        self.hints.as_ref()
    }

    /// Execute the query and return the result
    pub fn execute(&self, conn: &mut Client) -> Result<Vec<Row>, String> {
        let query = self.to_string();
        if self.timeout.is_some() {
            conn.execute(&self.timeout.as_ref().unwrap().to_string(), &[])
                .map_err(|e| e.to_string())?;
        }

        conn.query(&query, &[]).map_err(|e| e.to_string())
    }

    /// Get the plan corresponding to the query
    ///
    /// # Arguments
    ///
    /// * `conn`: The connection to the database
    /// * `analyze`: Whether to analyze the query
    pub fn get_plan(&self, conn: &mut Client, analyze: bool) -> Result<PlanRoot, String> {
        let prefix = format!(
            "EXPLAIN (FORMAT JSON, VERBOSE TRUE {})",
            if analyze { ", ANALYZE TRUE" } else { "" }
        );

        if self.timeout.is_some() {
            conn.execute(&self.timeout.as_ref().unwrap().to_string(), &[])
                .map_err(|e| e.to_string())?;
        }

        let query = format!("{}{}", prefix, self.to_string());
        let result = conn.query(&query, &[]).map_err(|e| e.to_string())?;

        let value: Option<Json<Vec<PlanRoot>>> = result.get(0).unwrap().get(0);

        Ok(value.unwrap().0[0].to_owned())
    }

    pub fn from_file(file_path: &str) -> Result<Query, String> {
        let sql = std::fs::read_to_string(file_path).map_err(|e| e.to_string())?;
        let query = Query::new(sql, None, None);
        Ok(query)
    }
}

impl Display for Query {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // let timeout = if self.timeout.is_some() {
        //     self.timeout.as_ref().unwrap().to_string()
        // } else {
        //     "".to_string()
        // };
        write!(
            f,
            "{}\n{}",
            // timeout,
            self.hints.as_ref().unwrap_or(&PgHintList::new()),
            self.sql
        )
    }
}

// Timeout in format postgres understands
#[derive(Clone)]

pub struct TimeOut {
    timeout: u64,
    timeout_type: DurationType,
}

impl TimeOut {
    pub fn new(timeout: u64, timeout_type: DurationType) -> Self {
        TimeOut {
            timeout,
            timeout_type,
        }
    }
}
impl Display for TimeOut {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SET statement_timeout = \'{}{}\';",
            self.timeout, self.timeout_type
        )
    }
}

#[derive(Clone)]

pub enum DurationType {
    Microseconds,
    Milliseconds,
    Seconds,
    Minutes,
    Hours,
    Days,
}

impl Display for DurationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DurationType::Microseconds => write!(f, "us"),
            DurationType::Milliseconds => write!(f, "ms"),
            DurationType::Seconds => write!(f, "s"),
            DurationType::Minutes => write!(f, "min"),
            DurationType::Hours => write!(f, "h"),
            DurationType::Days => write!(f, "d"),
        }
    }
}

#[cfg(test)]
mod test_query {
    use super::*;
    use crate::connector::establish_connection;
    use std::fs::File;

    #[test]
    fn test_timeout() {
        let timeout = TimeOut::new(1, DurationType::Seconds);

        let query = Query::new("SELECT * FROM name_basics".to_string(), None, Some(timeout));
        let mut conn = establish_connection("imdb", "postgres", "postgres", "localhost", "5432");
        let result = query.execute(&mut conn);
        // println!("{}", timeout);
        assert!(result.is_err(), "Query should have timed out but succeeded");
        match result {
            Err(e) => assert!(
                e.to_string().contains("statement timeout"),
                "Expected timeout error but got: {}",
                e
            ),
            Ok(_) => panic!("Query should have failed with timeout"),
        }
    }

    #[test]
    fn query_to_plan_test() {
        let sql = " SELECT * FROM name_basics limit 10";

        let mut conn = establish_connection("imdb", "postgres", "postgres", "localhost", "5432");

        let query = Query::new(sql.to_string(), None, None);

        let result = query.get_plan(&mut conn, false);

        assert!(result.is_ok(), "Query should not fail");

        println!("{:?}", result);
    }

    #[test]
    fn query_to_plan_analyze_test() {
        let sql = " SELECT * FROM name_basics limit 10";

        let query = Query::new(sql.to_string(), None, None);

        let mut conn = establish_connection("imdb", "postgres", "postgres", "localhost", "5432");

        let result = query.get_plan(&mut conn, true);

        println!("{:?}", result);
    }

    #[test]
    fn test_convert_sql_file_to_plan() {
        let mut conn = establish_connection("imdb", "postgres", "postgres", "localhost", "5432");
        let query = Query::from_file("resources/test_sql/q1.sql").unwrap();
        let plan = query.get_plan(&mut conn, false).unwrap();
        plan.save_json("resources/test_json/q1_test.json").unwrap();

        assert!(File::open("resources/test_json/q1_test.json").is_ok());
    }
}
