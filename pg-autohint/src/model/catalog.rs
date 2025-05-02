use postgres::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(PartialEq, Eq, Serialize)]
pub struct Catalog {
    pub tables: HashMap<String, Table>,
    pub indexes: HashMap<String, Index>, // index_name -> index
}

impl Catalog {
    pub fn new(conn: &mut Client) -> Self {
        let mut tables: HashMap<String, Table> = HashMap::new();
        let mut indexes: HashMap<String, Index> = HashMap::new();

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
                    indexes: vec![],
                },
            );
        });

        // Then collect all indexes
        let table_names = tables.keys().cloned().collect::<Vec<String>>();
        for table_name in table_names {
            let index_list = conn
                .query(
                    "SELECT * FROM pg_catalog.pg_indexes WHERE tablename = $1",
                    &[&table_name],
                )
                .unwrap();
            for index in index_list {
                let index_name: String = index.get("indexname");
                let index_def: String = index.get("indexdef");
                let columns: Vec<String> = index_def
                    .split("(")
                    .nth(1)
                    .unwrap()
                    .split(")")
                    .nth(0)
                    .unwrap()
                    .split(",")
                    .map(|s| s.trim().to_string())
                    .collect();
                indexes.insert(
                    index_name.clone(),
                    Index {
                        table_name: table_name.clone(),
                        columns,
                    },
                );
                tables
                    .get_mut(&table_name)
                    .unwrap()
                    .indexes
                    .push(index_name);
            }
        }

        Self { tables, indexes }
    }
}

#[derive(Hash, PartialEq, Eq, Serialize, Deserialize, Clone, Debug)]
pub struct Table {
    pub name: String,
    pub indexes: Vec<String>,
}

#[derive(Hash, PartialEq, Eq, Serialize, Deserialize, Clone, Debug)]
pub struct Index {
    pub table_name: String,
    pub columns: Vec<String>,
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
        assert_eq!(
            catalog.indexes.get("test_index"),
            Some(&Index {
                table_name: "test".to_string(),
                columns: vec!["id".to_string()],
            })
        );

        assert_eq!(catalog.tables.get("test").unwrap().indexes.len(), 1);

        conn.query("DROP INDEX test_index", &[]).unwrap();
        conn.query("DROP TABLE test", &[]).unwrap();
    }
}
