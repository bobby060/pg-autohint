/// External-facing API for postgres2sql

/// Provides api for converting JSON string (and json path) to SQL

/// Steps:
/// 1. Deserialize the JSON string into a Postgres plan
/// 2. Convert the Postgres plan to a datafusion AST
/// 3. Add hints


