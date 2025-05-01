//// DEPRECATED: Not used anymore. Proof of concept for converting physical plan back to SQL AST

use crate::model::postgresplan::*;
use sqlparser::{
    ast::helpers::attached_token::AttachedToken, ast::*, dialect::GenericDialect, parser,
};
use std::vec;

/// Trait to convert a plan node to a AST SetExpr
pub trait Visit {
    fn visit_plan_node(self) -> Result<SetExpr, String>;
}

impl Visit for PlanRoot {
    fn visit_plan_node(self) -> Result<SetExpr, String> {
        let expr = self.plan.visit_plan_node()?;

        // Placeholder for the AST
        if let SetExpr::Query(query) = expr {
            Ok(SetExpr::Query(query))
        } else {
            let query = Query {
                with: None,
                body: Box::new(expr),
                order_by: None,
                limit: None,
                limit_by: vec![],
                offset: None,
                fetch: None,
                locks: vec![],
                for_clause: None,
                settings: None,
                format_clause: None,
            };
            Ok(SetExpr::Query(Box::new(query)))
        }
    }
}

impl Visit for PlanNode {
    fn visit_plan_node(self) -> Result<SetExpr, String> {
        match self {
            PlanNode::SeqScan(scan) => ScanNode::SeqScan(scan).visit_plan_node(),
            PlanNode::IndexScan(scan) => ScanNode::IndexScan(scan).visit_plan_node(),
            PlanNode::Hash(hash) => hash.visit_plan_node(),
            PlanNode::HashJoin(join) => JoinNode::HashJoin(join).visit_plan_node(),
            PlanNode::MergeJoin(join) => JoinNode::MergeJoin(join).visit_plan_node(),
            PlanNode::Limit(limit) => limit.visit_plan_node(),
            PlanNode::Sort(sort) => sort.visit_plan_node(),
            PlanNode::Unique(unique) => unique.visit_plan_node(),
            PlanNode::Append(append) => SetNode::Append(append).visit_plan_node(),
            PlanNode::Gather(gather) => gather.visit_plan_node(),
            PlanNode::Aggregate(agg) => agg.visit_plan_node(),
            PlanNode::GatherMerge(gather_merge) => gather_merge.visit_plan_node(),
            PlanNode::NestedLoopJoin(join) => JoinNode::NestedLoopJoin(join).visit_plan_node(),
            PlanNode::IndexOnlyScan(scan) => ScanNode::IndexOnlyScan(scan).visit_plan_node(),
            PlanNode::Memoize(memoize) => memoize.visit_plan_node(),
            _ => Err("Node not implemented".to_string()),
        }
    }
}

impl Visit for Aggregate {
    fn visit_plan_node(self) -> Result<SetExpr, String> {
        // Ignore partial aggregates, since they are duplicates of the final aggregates

        fn aggregate_child(
            child: SetExpr,
            output: String,
            group_keys: Option<Vec<String>>,
        ) -> Result<SetExpr, String> {
            match &child {
                SetExpr::Select(select) => {
                    let mut select = select.to_owned();
                    select.projection = vec![SelectItem::UnnamedExpr(
                        Expr::from_str(output.as_str()).unwrap(),
                    )];

                    if let Some(group_keys) = group_keys {
                        select.group_by = GroupByExpr::Expressions(
                            group_keys
                                .iter()
                                .map(|key| Expr::from_str(key.as_str()).unwrap())
                                .collect(),
                            vec![],
                        );
                    }
                    Ok(SetExpr::Select(select))
                }
                // Really don't know how to handle an aggergate on a set yet, but in the union_all case, we ignore
                // Maybe convert to a select from a subquery?
                SetExpr::SetOperation { .. } => Ok(child),
                SetExpr::Query(query) => {
                    let mut query = query.to_owned();
                    query.body =
                        Box::new(aggregate_child(*query.body.to_owned(), output, group_keys)?);
                    Ok(SetExpr::Query(query))
                }

                _ => Err("Expected a select or setup statement".to_string()),
            }
        }

        if self.partial_mode == "Partial" {
            return Ok(self.children.unwrap()[0].to_owned().visit_plan_node()?);
        }
        let children = self.children.unwrap()[0].to_owned().visit_plan_node()?;

        aggregate_child(
            children,
            self.output.unwrap()[0].clone(),
            self.group_keys.clone(),
        )
    }
}

impl Visit for ScanNode {
    fn visit_plan_node(self) -> Result<SetExpr, String> {
        // tables
        let mut from: Vec<TableWithJoins> = vec![];
        // predicates
        let mut selection = match self.get_filter() {
            Some(filter) => Some(FromStr::from_str(&filter)?),
            None => None,
        };

        // parse table
        let table = build_base_table(self.get_relation_name(), self.get_alias())?;
        from.push(table);

        if let ScanNode::IndexOnlyScan(index_only_scan) = &self {
            let index_cond: Expr = FromStr::from_str(index_only_scan.index_cond.as_ref().unwrap())?;
            selection = Some(index_cond);
        }
        // build ast struct
        let select = Select {
            select_token: AttachedToken::empty(),
            distinct: None,
            projection: parse_projections(self.get_output().unwrap())?,
            into: None,
            from: from,
            group_by: GroupByExpr::Expressions(vec![], vec![]),
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

impl Visit for Limit {
    fn visit_plan_node(self) -> Result<SetExpr, String> {
        let limit = Expr::from_str(self.limit_rows.to_string().as_str())?;

        let child_expr = self.children.unwrap()[0].clone().visit_plan_node()?;

        if let SetExpr::Query(query) = child_expr {
            let mut query = query.to_owned();
            query.limit = Some(limit);
            return Ok(SetExpr::Query(query));
        }

        let query = Query {
            with: None,
            body: Box::new(child_expr),
            order_by: None,
            limit: Some(limit),
            limit_by: vec![],
            offset: None,
            fetch: None,
            locks: vec![],
            for_clause: None,
            settings: None,
            format_clause: None,
        };

        Ok(SetExpr::Query(Box::new(query)))
    }
}

impl Visit for Hash {
    // Hash node is a noop, just return the child
    fn visit_plan_node(self) -> Result<SetExpr, String> {
        self.children.unwrap()[0].to_owned().visit_plan_node()
    }
}

impl Visit for Memoize {
    fn visit_plan_node(self) -> Result<SetExpr, String> {
        self.children.unwrap()[0].to_owned().visit_plan_node()
    }
}

impl Visit for JoinNode {
    fn visit_plan_node(self) -> Result<SetExpr, String> {
        let mut children_exprs = vec![];
        for child in self.get_children().unwrap() {
            let child_expr = child.visit_plan_node()?;
            children_exprs.push(child_expr);
        }

        // Add join condition to predicate
        // make project correct
        // merge child table

        // Assuming both children are SELECTs...
        // TODO: does postgres join always have 2 children?
        assert!(children_exprs.len() == 2);

        let left_select = children_exprs[0].as_select().unwrap();
        let right_select = children_exprs[1].as_select().unwrap();

        let mut select = left_select.to_owned();

        select.projection = parse_projections(self.get_output().unwrap())?;

        for table in right_select.from.iter() {
            select.from.push(table.to_owned());
        }

        // How to handle different join types?

        // combine table predicates

        select.selection = if let Some(selection) = select.selection {
            if let Some(right_selection) = right_select.selection.as_ref() {
                Some(Expr::BinaryOp {
                    left: Box::new(selection),
                    right: Box::new(right_selection.to_owned()),
                    op: BinaryOperator::And,
                })
            } else {
                Some(selection)
            }
        } else {
            right_select.selection.as_ref().map(|expr| expr.to_owned())
        };

        // Combine join predicate with selection

        if let Some(condition) = self.get_condition() {
            match select.selection {
                Some(selection) => {
                    select.selection = Some(Expr::BinaryOp {
                        left: Box::new(selection),
                        right: Box::new(Expr::from_str(condition.as_str())?),
                        op: BinaryOperator::And,
                    })
                }
                None => select.selection = Some(Expr::from_str(condition.as_str())?),
            }
        }

        // Combine having
        select.having = if let Some(having) = select.having {
            if let Some(right_having) = right_select.having.as_ref() {
                Some(Expr::BinaryOp {
                    left: Box::new(having),
                    right: Box::new(right_having.to_owned()),
                    op: BinaryOperator::And,
                })
            } else {
                Some(having)
            }
        } else {
            right_select.having.as_ref().map(|expr| expr.to_owned())
        };

        Ok(SetExpr::Select(Box::new(select)))
    }
}

// NOTE: the Output field for a sorted plan includes the sort key, even if not part of original query output
impl Visit for Sort {
    fn visit_plan_node(self) -> Result<SetExpr, String> {
        // Convert text order by keys to OrderByExpr
        let order_by_items: Vec<OrderByExpr> = self
            .sort_keys
            .iter()
            .map(|key| OrderByExpr {
                expr: parse_expr(key).unwrap(),
                options: OrderByOptions {
                    asc: match key.ends_with("DESC") {
                        true => Some(false),
                        false => Some(true),
                    },
                    nulls_first: None,
                },
                with_fill: None,
            })
            .collect();

        let order_by = OrderBy {
            kind: OrderByKind::Expressions(order_by_items),
            interpolate: None,
        };

        let child_expr = self.children.unwrap()[0].clone().visit_plan_node()?;

        // If already a query, just add order by
        if let SetExpr::Query(query) = child_expr {
            let mut query = query.to_owned();
            query.order_by = Some(order_by);
            return Ok(SetExpr::Query(query));
        }

        // Otherwise wrap in a query
        let query = Query {
            with: None,
            body: Box::new(child_expr),
            order_by: Some(order_by),
            limit: None,
            limit_by: vec![],
            offset: None,
            fetch: None,
            locks: vec![],
            for_clause: None,
            settings: None,
            format_clause: None,
        };

        Ok(SetExpr::Query(Box::new(query)))
    }
}

impl Visit for SetNode {
    fn visit_plan_node(self) -> Result<SetExpr, String> {
        // Recursive helper for building set operations, lets you have more than two children
        fn recursive_set_op_builder(
            children: Vec<SetExpr>,
            op: SetOperator,
            quant: SetQuantifier,
        ) -> Result<SetExpr, String> {
            Ok(SetExpr::SetOperation {
                op: op,
                set_quantifier: quant,
                left: Box::new(children[0].clone()),
                right: {
                    if children.len() == 2 {
                        Box::new(children[1].clone())
                    } else {
                        Box::new(recursive_set_op_builder(children[1..].to_vec(), op, quant)?)
                    }
                },
            })
        }

        let children_exprs = self
            .get_children()
            .unwrap()
            .iter()
            .map(|child| child.to_owned().visit_plan_node().unwrap())
            .collect();

        Ok(recursive_set_op_builder(
            children_exprs,
            self.get_operator(),
            SetQuantifier::None,
        )?)
    }
}

impl Visit for Unique {
    fn visit_plan_node(self) -> Result<SetExpr, String> {
        /// make the child set operation distinct
        /// A Query needs this performed on its own setop
        fn make_distinct(child: SetExpr) -> Result<SetExpr, String> {
            match child {
                SetExpr::Select(select) => {
                    let mut select = select.to_owned();
                    select.distinct = Some(Distinct::Distinct);
                    Ok(SetExpr::Select(select))
                }
                SetExpr::Query(query) => {
                    let mut query = query.to_owned();
                    query.body = Box::new(make_distinct(*query.body.to_owned())?);
                    Ok(SetExpr::Query(query))
                }
                SetExpr::SetOperation {
                    op,
                    set_quantifier: _,
                    left,
                    right,
                } => {
                    let set_operation = SetExpr::SetOperation {
                        op: op.to_owned(),
                        set_quantifier: SetQuantifier::Distinct,
                        left: left.to_owned(),
                        right: right.to_owned(),
                    };
                    Ok(set_operation)
                }
                _ => Err("Not supported!".to_string()),
            }
        }
        let child = self.children.unwrap()[0].to_owned().visit_plan_node()?;

        make_distinct(child)
    }
}

impl Visit for Gather {
    fn visit_plan_node(self) -> Result<SetExpr, String> {
        match self.children {
            Some(children) => Ok(children
                .get(0)
                .ok_or_else(|| "Gather has no children, expected 1".to_string())?
                .clone()
                .visit_plan_node()?),
            None => Err("Gather has no children, expected 1".to_string()),
        }
    }
}

impl Visit for GatherMerge {
    fn visit_plan_node(self) -> Result<SetExpr, String> {
        match self.children {
            Some(children) => Ok(children
                .get(0)
                .ok_or_else(|| "GatherMerge has no children, expected 1".to_string())?
                .clone()
                .visit_plan_node()?),
            None => Err("GatherMerge has no children, expected 1".to_string()),
        }
    }
}

fn parse_projections(output: Vec<String>) -> Result<Vec<SelectItem>, String> {
    Ok(output
        .iter()
        .map(|x| SelectItem::UnnamedExpr(parse_expr(x).map_err(|e| e.to_string()).unwrap()))
        .collect())
}

/// Build a base table with no joins
///
/// # Arguments
///
/// * `relation_name`: The name of the relation to build the table from
/// * `alias`: The alias of the table
///
/// # Returns a TableWithJoins struct
fn build_base_table(
    relation_name: Option<String>,
    alias: String,
) -> Result<TableWithJoins, String> {
    Ok(TableWithJoins {
        joins: vec![],
        relation: TableFactor::Table {
            name: match &relation_name {
                Some(name) => ObjectName::from_str(name)?,
                None => ObjectName::from_str(&alias)?,
            },
            alias: {
                let ident = match parse_expr(alias.as_str()) {
                    Ok(Expr::Identifier(ident)) => ident,
                    _ => {
                        return Err(format!(
                            "Failed to parse alias: expected an identifier, but got '{}'",
                            alias
                        ))
                    }
                };
                Some(TableAlias {
                    name: ident,
                    columns: vec![],
                })
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
    })
}

/// this test module is used to display reference asts
/// only used during development
#[cfg(test)]
mod show_ref_ast {
    use super::*;
    use sqlparser::parser::Parser;
    // helper func to display reference ast
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
            }
            Err(e) => println!("Error parsing SQL: {}", e),
        }
    }

    #[test]
    #[ignore]
    // this function is meant for you to see what the ast is supposed to look like conveniently
    fn show_ref_seq_scan() {
        ref_helper("SELECT title_basics.tconst, titletype, primarytitle FROM title_basics WHERE (runtimeminutes < 25)");
    }

    #[test]
    #[ignore]
    fn show_ref_seq_scan_alias() {
        ref_helper("SELECT t1.tconst a, titlebasics.titletype b, primarytitle c FROM title_basics t1 WHERE (runtimeminutes < 25)");
        todo!(
            "aliases are not handled now, need to see when are aliases outputed by postgres plan!"
        );
    }
}

/// tests for impl Visit for PlanNodes
#[cfg(test)]
mod test_visit_nodes {
    use super::*;

    // test visit_plan_node for SeqScan with projection, predicate, table alias, compound column identifiers
    #[test]
    fn test_visit_seq_scan() {
        // Guide to test operator's visit:
        // first define the test query, notice some expressions should be within parenthesis if the PlanNode tree has it,
        // an expression with parenthesis are wrapped with Expr::Nested in ast.
        // (p.s. there seemed to be no case that the output columns has aliases, we can add them if needed)
        let test_query = "SELECT title_basics.tconst, titletype, primarytitle FROM title_basics AS t1 WHERE (runtimeminutes < 25)";
        let test_ast = parse_query(test_query).unwrap().body;
        // constuct the PlanNode as if it was created via postgres2plan, also notice the parenthesis
        let scan_node = ScanNode::SeqScan(SeqScan {
            parent_relationship: Some("Outer".to_string()),
            relation_name: Some("title_basics".to_string()),
            alias: "t1".to_string(),
            filter: Some("(runtimeminutes < 25)".to_string()),
            actual_rows: None,
            plan_rows: None,
            output: Some(vec![
                "title_basics.tconst".to_string(),
                "titletype".to_string(),
                "primarytitle".to_string(),
            ]), // notice that the postgres plan output never uses *, but list all cols
        });
        // call to your visit_plan_node here and compare against the test_ast
        let result = scan_node.visit_plan_node();
        let result = result.unwrap();
        assert_eq!(
            result, *test_ast,
            "Mismatch: Parsed Result: {:#?}, Test AST: {:#?}",
            result, test_ast
        );
        // if the ast is identical, the output SQL query are semantically equivalent,
        // the only mismatches are minors like the converted will always have
        // 'AS' when specifying aliases, while what the user has written may not.
    }

    // visit gather node should ignore it, returning the visit result of its only children
    #[test]
    fn test_visit_gather() {
        let test_query =
            "SELECT tconst FROM title_basics AS title_basics WHERE (runtimeminutes < 25)";
        let test_ast = parse_query(test_query).unwrap().body;
        let scan_node = SeqScan {
            parent_relationship: Some("Outer".to_string()),
            relation_name: Some("title_basics".to_string()),
            actual_rows: None,
            plan_rows: None,
            alias: "title_basics".to_string(),
            filter: Some("(runtimeminutes < 25)".to_string()),
            output: Some(vec!["tconst".to_string()]),
        };
        let gather_node = Gather {
            children: Some(vec![PlanNode::SeqScan(scan_node)]),
            parent_relationship: None,
            num_workers: Some(1),
            output: None,
        };
        let result = gather_node.visit_plan_node();
        let result = result.unwrap();
        assert_eq!(
            result, *test_ast,
            "Mismatch: Parsed Result: {:#?}, Test AST: {:#?}",
            result, test_ast
        );
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
            _ => Err(format!("Failed to parse value: '{}'", value)),
        }
    }
}

/// A name of a table, view, custom type, etc., possibly multi-part, i.e. db.schema.obj
impl FromStr for ObjectName {
    fn from_str(name: &str) -> Result<Self, String> {
        match parse_expr(name) {
            // if is simple identifier, e.g. table1
            Ok(Expr::Identifier(ident)) => Ok(ObjectName::from(vec![ident])),
            // if is compound identifier, e.g. db_schema.table1
            Ok(Expr::CompoundIdentifier(idents)) => Ok(ObjectName::from(idents)),
            _ => Err(format!(
                "Failed to parse identifier: expected an identifier, but got '{}'",
                name
            )),
        }
    }
}

/// An identifier, decomposed into its value or character data and the quote style.
impl FromStr for Ident {
    fn from_str(name: &str) -> Result<Self, String> {
        // only accept one identifier ?
        match parse_expr(name) {
            Ok(Expr::Identifier(ident)) => Ok(ident),
            _ => Err(format!(
                "Failed to parse identifier, expected an identifier, but got '{}'",
                name
            )),
        }
    }
}

/// helper function used for parsing a string expression into a sqlparser::ast::Expr
fn parse_expr(expr: &str) -> Result<Expr, parser::ParserError> {
    let parser = parser::Parser::new(&GenericDialect);
    let result = parser.try_with_sql(expr);
    let mut parser = result.unwrap();
    let _token = parser.token_at(0).clone();
    parser.parse_expr()
}

/// helper function used for parsing a SQL query string into a Box<sqlparser::ast::Query>
pub fn parse_query(query: &str) -> Result<Box<Query>, parser::ParserError> {
    let parser = parser::Parser::new(&GenericDialect);
    let result = parser.try_with_sql(query);
    let mut parser = result.unwrap();
    let _token = parser.token_at(0).clone();
    parser.parse_query()
}
// test module for impl FromStr trait for string expressions in PlanNodes
// TODO: most of the tests does not do assert! yet, they simply print the converted Expr!
#[cfg(test)]
mod test_from_str {
    use super::*;
    #[test]
    fn test_equality_expr() {
        let expr = parse_expr("(title_principals.tconst = title_basics.tconst)").unwrap();
        println!("{:#?}", expr);
        assert!(matches!(expr, Expr::Nested(inner) if matches!(*inner, Expr::BinaryOp { .. })));
    }

    #[test]
    fn test_single_quoted_string_expr() {
        let expr = parse_expr("(category = 'actor'::text)").unwrap();
        println!("{:#?}", expr);
    }

    #[test]
    fn test_value() {
        let expr = parse_expr("'value1'").unwrap();
        println!("{:#?}", expr);

        let expr = parse_expr("123").unwrap();
        println!("{:#?}", expr);
    }

    #[test]
    fn test_simple_identifier() {
        let expr = parse_expr("title_basics").unwrap();
        println!("{:#?}", expr);
    }

    #[test]
    fn test_simple_identifier_with_schema_name() {
        let expr = parse_expr("db_schema.title_basics").unwrap();
        println!("{:#?}", expr);
    }

    #[test]
    fn test_compound_identifier() {
        let expr = parse_expr("title_principals.tconst").unwrap();
        println!("{:#?}", expr);
        assert!(matches!(expr, Expr::CompoundIdentifier { .. }));
    }

    #[test]
    fn test_type_cast_expr() {
        let expr = parse_expr("(name_basics.nconst)::text)").unwrap();
        println!("{:#?}", expr);
        assert!(matches!(expr, Expr::Cast { .. }));

        let expr = parse_expr("((startyear)::numeric > $2)").unwrap();
        println!("{:#?}", expr);
    }

    #[test]
    fn test_and_expr() {
        let expr = parse_expr("((category = 'actor'::text) AND (job = 'actor'::text))").unwrap();
        println!("{:#?}", expr);
    }

    #[test]
    fn test_pg_like_match_expr() {
        let expr = parse_expr("(genres ~~ '%Comedy%'::text)").unwrap();
        println!("{:#?}", expr);
    }

    /// for matching this placeholder, we need to parse the "InitPlan .. (return $1)" statement in "Subplan Name" field
    #[test]
    fn test_subquery_placeholder_expr() {
        let expr = parse_expr("((runtimeminutes)::numeric > $1)").unwrap();
        println!("{:#?}", expr);
    }

    #[test]
    fn test_is_null_expr() {
        let expr = parse_expr("(runtimeminutes IS NOT NULL)").unwrap();
        println!("{:#?}", expr);

        let expr = parse_expr("(runtimeminutes IS NULL)").unwrap();
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
        let expr = parse_expr("title_basics.primarytitle DESC").unwrap();
        println!("{:#?}", expr);
    }
}

#[cfg(test)]
mod test_visit {
    use super::*;
    use crate::connector::*;
    use crate::model::query::Query as PgQuery;
    use crate::test_utils::*;
    use std::path::Path;

    // #[test]
    fn test_input_plan(input_path: &str) {
        let plan_root = PlanRoot::from_json(input_path).expect("Failed to parse input");
        let new_sql = plan_root.to_sql().expect("Failed to parse input");

        let sql_path = input_path.replace("json", "sql");
        let original_sql = std::fs::read_to_string(Path::new(sql_path.as_str()))
            .expect("Failed to read original sql file");

        println!("Original SQL: {}", &original_sql);

        println!("New SQL: {}", &new_sql);

        correctness_test(
            "imdb",
            &original_sql,
            &new_sql,
            new_sql.contains("ORDER BY"),
        );
    }

    #[test]
    fn test_order_by_visit() {
        test_input_plan("resources/test_json/simple_orderby.json");
        test_input_plan("resources/test_json/simple_orderby_desc.json");
    }

    #[test]
    fn test_limit() {
        test_input_plan("resources/test_json/q4.json");
    }

    #[test]
    fn test_union_all() {
        test_input_plan("resources/test_json/union_all.json");
    }

    // #[test]
    // fn test_values_scan() {
    //     test_input_plan("resources/test_json/values_scan.json");
    // }

    #[test]
    fn test_simple_index_scan() {
        test_input_plan("resources/test_json/simple_index_scan.json");
    }

    #[test]
    fn test_group_by_visit() {
        test_input_plan("resources/test_json/simple_groupby.json");
    }

    #[test]
    fn test_limit_visit() {
        test_input_plan("resources/test_json/q4.json");
    }

    #[test]
    fn test_nlj() {
        test_input_plan("resources/test_json/q7.json");
    }

    // fails
    // #[test]
    // fn test_union() {
    //     test_input_plan("resources/test_json/q9.json");
    // }

    #[test]
    // This takes a long time to run - try to replace with new query?
    fn test_hash_join() {
        test_input_plan("resources/test_json/q3.json");
    }

    #[test]
    fn test_not_visited() {
        let plan = ResultNode {
            parent_relationship: None,
            subplan_name: None,
            output: None,
            filter: None,
        };
        assert_eq!(
            PlanNode::ResultNode(plan).visit_plan_node(),
            Err("Node not implemented".to_string())
        );
    }

    #[test]
    fn test_visit_root() {
        let mut conn = establish_connection("imdb", "postgres", "postgres", "localhost", "5432");

        let query = PgQuery::new("SELECT * FROM title_basics".to_string(), None, None);
        let root = query.get_plan(&mut conn, false).unwrap();
        let ast = root.visit_plan_node();
        println!("{:#?}", ast);
    }

    #[test]
    #[ignore]
    fn test_order_by_not_in_result() {
        test_input_plan("resources/test_json/order_by_bad_col.json");
    }

    // fails
    // #[test]
    // fn test_visit_index_only_scan() {
    //     test_input_plan("resources/test_json/q10.json");
    // }
}
