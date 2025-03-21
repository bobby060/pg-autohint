

Use Diesel to generate the models from the database.
https://diesel.rs/guides/getting-started


I needed to sudo apt install libpq-dev
`cargo install diesel_cli --no-default-features --features postgres
`
Migrating:
https://www.rharriso.com/connecting-diesel-to-an-existing-postgresql-server.html

Create schema with
`diesel print-schema > src/schema.rs`
Create model with
`diesel_ext model > src/models.rs`

- add primary keys to setup script
- add test_utils.rs with correctness test (needs to be updated to allow unordered queries)
- change test inputs to resources/ Each corresponding sql file needs its own json
- Change postgres2plan tests to use all input files
- Define interface for postgres2sql.rs
- Define interface for plan2ast.rs