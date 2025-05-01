use postgres::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(PartialEq, Eq, Serialize)]
struct Catalog {
    tables: HashMap<String, Table>,
    indexes: HashMap<String, String>, // index_name -> table_name
}

impl Catalog {
    fn new(conn: &mut Client) -> Self {
        let mut tables: HashMap<String, Table> = HashMap::new();
        let mut indexes: HashMap<String, String> = HashMap::new();

        // First collect all tables
        conn.query(
            "SELECT *
FROM pg_catalog.pg_tables
WHERE schemaname != 'pg_catalog' AND
    schemaname != 'information_schema'",
            &[],
        )
        .unwrap()
        .iter()
        .for_each(|row| {
            let table_name: String = row.get("tablename");
            tables.insert(
                table_name.clone(),
                Table {
                    name: table_name,
                    columns: vec![],
                },
            );
        });

        // Then collect all indexes
        for table_name in tables.keys() {
            let index_list = conn
                .query(
                    "SELECT * FROM pg_catalog.pg_indexes WHERE tablename = $1",
                    &[table_name],
                )
                .unwrap();
            for index in index_list {
                let index_name: String = index.get("indexname");
                indexes.insert(index_name, table_name.clone());
            }
        }

        Self { tables, indexes }
    }
}

#[derive(Hash, PartialEq, Eq, Serialize, Deserialize, Clone)]
struct Table {
    name: String,
    columns: Vec<String>,
}

#[cfg(test)]
mod test_catalog {
    use super::*;
    use crate::connector::establish_connection;

    #[test]
    fn test_catalog() {
        let mut conn = establish_connection("imdb", "postgres", "postgres", "localhost", "5432");
        conn.query("CREATE TABLE test (id INT, name VARCHAR)", &[])
            .unwrap();
        conn.query("CREATE INDEX test_index ON test (id)", &[])
            .unwrap();
        let catalog = Catalog::new(&mut conn);
        assert!(catalog.tables.contains_key("test"));
        assert!(catalog.indexes.contains_key("test_index"));
        assert_eq!(catalog.indexes.get("test_index"), Some(&"test".to_string()));

        conn.query("DROP INDEX test_index", &[]).unwrap();
        conn.query("DROP TABLE test", &[]).unwrap();
    }
}
