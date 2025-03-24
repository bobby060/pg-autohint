use std::vec;

use crate::postgres2plan::*;
use sqlparser::ast::helpers::attached_token::AttachedToken;
use sqlparser::{ast::{self, *}, parser};
use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;
/// Given a Postgres plan, convert it to a datafusion AST
///
///
///
///
pub fn plan2ast(plan: PlanNode) -> Result<Query, String> {
    // 1. Build body (SetExpr)
    // if let PlanNode::Limit {
    //     limit_rows,
    //     children,
    //     ..
    // } = plan
    // {
    //     let limit = LimitClause::from_plan_node(plan)?;
    //     let plan = children.unwrap()[0];
    // }

    // 1.1 If select:
    // Call build_select
    let _expr = plan.visit_plan_node()?;

    // 1.2 If set, build of children recursively

    // 2. Build order by

    // 3. Build limit

    // Placeholder for the AST
    let ast = Query {
        with: None,
        body: Box::new(SetExpr::Select(Box::new(Select {
            select_token: AttachedToken::empty(),
            distinct: None,
            projection: vec![],
            into: None,
            from: vec![],
            group_by: GroupByExpr::All(vec![]),
            top: None,
            top_before_distinct: false,
            lateral_views: vec![],
            prewhere: None,
            selection: None,
            cluster_by: vec![],
            connect_by: None,
            distribute_by: vec![],
            sort_by: vec![],
            having: None,
            named_window: vec![],
            qualify: None,
            window_before_qualify: false,
            flavor: SelectFlavor::Standard,
            value_table_mode: None,
        }))),
        order_by: None,
        limit: None,
        limit_by: Vec::new(),
        offset: None,
        fetch: None,
        locks: Vec::new(),
        for_clause: None,
        settings: None,
        format_clause: None,
    };
    Ok(ast)
}

trait Visit {
    fn visit_plan_node(self) -> Result<SetExpr, String>;
}

impl Visit for PlanNode {
    fn visit_plan_node(self) -> Result<SetExpr, String> {
        // Conceptuallly, will need to perform the following:
        // 1. Extract the final projection
        // 2. Create an expression tree for all filters in the scans
        // 3. Add each table to the from clause, including joins

        match self {
            PlanNode::SeqScan(scan) => scan.visit_plan_node(),
            PlanNode::IndexScan(scan) => scan.visit_plan_node(),
            PlanNode::Hash(hash) => hash.visit_plan_node(),
            PlanNode::HashJoin(join) => join.visit_plan_node(),
            PlanNode::MergeJoin(join) => join.visit_plan_node(),
            PlanNode::Limit(limit) => limit.visit_plan_node(),
            PlanNode::Sort(sort) => sort.visit_plan_node(),
            PlanNode::Unique(unique) => unique.visit_plan_node(),
            PlanNode::Append(append) => append.visit_plan_node(),
            PlanNode::Gather(gather) => gather.visit_plan_node(),
            _ => Err("Not implemented".to_string()),
        }
    }
}

impl Visit for Aggregate {
    fn visit_plan_node(self) -> Result<SetExpr, String> {
        Err("Not implemented".to_string())
    }
}

impl Visit for SeqScan {
    fn visit_plan_node(self) -> Result<SetExpr, String> {
        // TODO: FIX PROJECTION!!!
        let projection: Vec<SelectItem> = vec![];
        let mut from: Vec<TableWithJoins> = vec![];
        let mut selection = None;
        let group_by: GroupByExpr = GroupByExpr::Expressions(vec![], vec![]);

        if let Some(filter) = self.filter {
            let filter: Expr = FromStr::from_str(&filter)?;
            selection = Some(filter);
        }

        let table = TableWithJoins {
            joins: vec![],
            relation: TableFactor::Table {
                name: ObjectName::from_str(&self.relation_name)?,
                alias: if let Some(alias) = self.alias {
                    Some(TableAlias {
                        name: Ident::from_str(&alias)?,
                        columns: vec![], // TODO: add columns
                    })
                } else {
                    None
                },
                args: None,
                with_hints: vec![],
                version: None,
                with_ordinality: false,
                partitions: vec![],
                json_path: None,
                sample: None,
                index_hints: vec![],
            },
        };

        from.push(table);

        // TODO: FIX PROJECTION!!!
        let select = Select {
            select_token: AttachedToken::empty(),
            distinct: None,
            projection: projection,
            into: None,
            from: from,
            group_by: group_by,
            top: None,
            top_before_distinct: false,
            lateral_views: vec![],
            prewhere: None,
            selection: selection,
            cluster_by: vec![],
            connect_by: None,
            distribute_by: vec![],
            sort_by: vec![],
            having: None,
            named_window: vec![],
            qualify: None,
            window_before_qualify: false,
            flavor: SelectFlavor::Standard,
            value_table_mode: None,
        };

        Ok(SetExpr::Select(Box::new(select)))
    }
}

impl Visit for IndexScan {
    fn visit_plan_node(self) -> Result<SetExpr, String> {
        Err("Not implemented".to_string())
    }
}

impl Visit for Hash {
    fn visit_plan_node(self) -> Result<SetExpr, String> {
        Err("Not implemented".to_string())
    }
}

impl Visit for HashJoin {
    fn visit_plan_node(self) -> Result<SetExpr, String> {
        Err("Not implemented".to_string())
    }
}

impl Visit for MergeJoin {
    fn visit_plan_node(self) -> Result<SetExpr, String> {
        Err("Not implemented".to_string())
    }
}

impl Visit for Limit {
    fn visit_plan_node(self) -> Result<SetExpr, String> {
        Err("Not implemented".to_string())
    }
}

impl Visit for Sort {
    fn visit_plan_node(self) -> Result<SetExpr, String> {
        Err("Not implemented".to_string())
    }
}

impl Visit for Unique {
    fn visit_plan_node(self) -> Result<SetExpr, String> {
        Err("Not implemented".to_string())
    }
}

impl Visit for Append {
    fn visit_plan_node(self) -> Result<SetExpr, String> {
        Err("Not implemented".to_string())
    }
}

impl Visit for Gather {
    fn visit_plan_node(self) -> Result<SetExpr, String> {
        Err("Not implemented".to_string())
    }
}

#[cfg(test)]
mod test_visit_nodes {
    use super::*;

    fn ref_helper(sql: &str) {
        let dialect = GenericDialect {};
        match Parser::parse_sql(&dialect, sql) {
            Ok(ast) => {
                println!("Parsed successfully: {:#?}", ast);
                // write ast to a file as well
                println!("{:#?}", ast);

                // back to original sql
                for stmt in ast {
                    println!("{}", stmt.to_string());
                }
            },
            Err(e) => println!("Error parsing SQL: {}", e),
        }
    }

    #[test]
    fn show_ref_seq_scan() {
        ref_helper("SELECT * FROM title_basics WHERE runtimeminutes < 25");
    }

    #[test]
    fn test_visit_seq_scan() {
        let refsol = "SELECT * FROM title_basics WHERE runtimeminutes < 25";
        let scan_node = SeqScan{
            parent_relationship: Some("Outer".to_string()),
            relation_name: "title_basics".to_string(),
            alias: None,
            filter: Some("(runtimeminutes < 25)".to_string()),
        };
        let result = scan_node.visit_plan_node();
        let result = result.unwrap();
        println!("visit seq scan result:\n{:#?}", result);
        println!("{}", result.to_string());
        assert!(refsol == result.to_string());
    }
}

trait FromStr: Sized {
    fn from_str(s: &str) -> Result<Self, String>;
}

/// for any expr
impl FromStr for Expr {
    fn from_str(expr: &str) -> Result<Self, String> {
        parse_expr(expr).map_err(|e| e.to_string())
    }
}

/// Primitive SQL values such as number and string
impl FromStr for Value {
    fn from_str(value: &str) -> Result<Self, String> {
        match parse_expr(value) {
            Ok(Expr::Value(v)) => Ok(v.into()),
            _ => Err(format!("Failed to parse value: '{}'", value))
        }
    }
}

/// A name of a table, view, custom type, etc., possibly multi-part, i.e. db.schema.obj
impl FromStr for ObjectName {
    fn from_str(name: &str) -> Result<Self, String> {
        match parse_expr(name) {
            // if is simple identifier, e.g. table1
            Ok(Expr::Identifier(ident)) => {
                Ok(ObjectName::from(
                    vec![ident]
                ))
            },
            // if is compound identifier, e.g. db_schema.table1
            Ok(Expr::CompoundIdentifier(idents)) => {
                Ok(ObjectName::from(idents))
            },
            _ => {
                Err(format!(
                    "Failed to parse identifier: expected an identifier, but got '{}'",
                    name
                ))
            }
        }
    }
}

/// An identifier, decomposed into its value or character data and the quote style.
impl FromStr for Ident {
    fn from_str(name: &str) -> Result<Self, String> {
        // only accept one identifier ?
        match parse_expr(name) {
            Ok(Expr::Identifier(ident)) => Ok(ident),
            _ => Err(format!("Failed to parse identifier, expected an identifier, but got '{}'", name))
        }
    }
}

/// function used for parsing a string expression into a sqlparser::ast::Expr
fn parse_expr(expr: &str) -> Result<Expr, parser::ParserError> {
    let parser= Parser::new(&GenericDialect);
    let result = parser.try_with_sql(expr);
    let mut parser = result.unwrap();
    let _token = parser.token_at(0).clone();
    parser.parse_expr()
}

#[cfg(test)]
mod test_from_str {
    use super::*;

    #[test]
    fn test_equality_expr() {
        let expr = parse_expr("(title_principals.tconst = title_basics.tconst)")
        .unwrap();
        println!("{:#?}", expr);
        assert!(matches!(expr, Expr::Nested(inner) if matches!(*inner, Expr::BinaryOp { .. })));
    }

    #[test]
    fn test_single_quoted_string_expr() {
        let expr = parse_expr("(category = 'actor'::text)")
        .unwrap();
        println!("{:#?}", expr);
    }

    #[test]
    fn test_value() {
        let expr = parse_expr("'value1'")
        .unwrap();
        println!("{:#?}", expr);

        let expr = parse_expr("123")
        .unwrap();
        println!("{:#?}", expr);
    }

    #[test]
    fn test_simple_identifier() {
        let expr = parse_expr("title_basics")
        .unwrap();
        println!("{:#?}", expr);
    }
    
    #[test]
    fn test_simple_identifier_with_schema_name() {
        let expr = parse_expr("db_schema.title_basics")
        .unwrap();
        println!("{:#?}", expr);
    }

    #[test]
    fn test_compound_identifier() {
        let expr = parse_expr("title_principals.tconst")
        .unwrap();
        println!("{:#?}", expr);
        assert!(matches!(expr, Expr::CompoundIdentifier{..}));
    }

    #[test]
    fn test_type_cast_expr() {
        let expr = parse_expr("(name_basics.nconst)::text)")
        .unwrap();
        println!("{:#?}", expr);
        assert!(matches!(expr, Expr::Cast { .. }));

        let expr = parse_expr("((startyear)::numeric > $2)")
        .unwrap();
        println!("{:#?}", expr);
    }

    #[test]
    fn test_and_expr() {
        let expr = parse_expr("((category = 'actor'::text) AND (job = 'actor'::text))")
        .unwrap();
        println!("{:#?}", expr);
    }

    #[test]
    fn test_pg_like_match_expr() {
        let expr = parse_expr("(genres ~~ '%Comedy%'::text)")
        .unwrap();
        println!("{:#?}", expr);
    }

    /// for matching this placeholder, we need to parse the "InitPlan .. (return $1)" statement in "Subplan Name" field
    #[test]
    fn test_subquery_placeholder_expr() {
        let expr = parse_expr("((runtimeminutes)::numeric > $1)")
        .unwrap();
        println!("{:#?}", expr);
    }

    #[test]
    fn test_is_null_expr() {
        let expr = parse_expr("(runtimeminutes IS NOT NULL)")
        .unwrap();
        println!("{:#?}", expr);

        let expr = parse_expr("(runtimeminutes IS NULL)")
        .unwrap();
        println!("{:#?}", expr);
    }

    #[test]
    fn test_complex_expr() {
        let expr = parse_expr("((tb.startyear > 2000) OR ((r.num_votes > 1000000) AND (tc.directors !~~ '%Tom%'::text)))")
        .unwrap();
        println!("{:#?}", expr);
    }

    /// In this case, the parser IGNORES the DESC suffix, we need to parse Sort Key field ourselfs, handling DESC
    #[test]
    fn fail_test_sort_key_desc() {
        let expr = parse_expr("title_basics.primarytitle DESC")
        .unwrap();
        println!("{:#?}", expr);
    }
}