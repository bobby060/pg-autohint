use crate::postgres2plan::PlanNode;
use sqlparser::ast::helpers::attached_token::AttachedToken;
use sqlparser::ast::query::LimitClause;
use sqlparser::ast::*;
use sqlparser::tokenizer::Span;
/// Given a Postgres plan, convert it to a datafusion AST
///
///
///
///
pub fn plan2ast(plan: PlanNode) -> Result<Query, String> {
    // 1. Build body (SetExpr)
    if let PlanNode::Limit {
        limit_rows,
        children,
        ..
    } = plan
    {
        let limit = LimitClause::from_plan_node(plan)?;
        let plan = children.unwrap()[0];
    }

    // 1.1 If select:
    // Call build_select
    let select = Select::from_plan_node(plan)?;

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

trait FromPlanNode {
    fn from_plan_node(plan: PlanNode) -> Result<Self, String>;
}

impl FromPlanNode for LimitClause {
    fn from_plan_node(plan: PlanNode) -> Result<Self, String> {
        if let PlanNode::Limit { limit_rows, .. } = plan {
            // Dont support clickhouse varient, only
            let limit = Some(Expr::Value(ValueWithSpan {
                value: Value::Number(limit_rows.to_string(), true),
                span: Span::empty(),
            }));
            Ok(LimitClause { limit })
        } else {
            Err(format!("Unsupported plan node: {:?}", plan))
        }
    }
}

impl FromPlanNode for Select {
    fn from_plan_node(plan: PlanNode) -> Self {
        let projection: Vec<SelectItem> = vec![];
        let from: Vec<TableWithJoins> = vec![];
        let group_by: GroupByExpr = GroupByExpr::All(vec![]);
        let sort_by: Vec<Expr> = vec![];
        let having: Option<Expr> = None;

        /// Conceptuallly, will need to perform the following:
        /// 1. Extract the final projection
        /// 2. Create an expression tree for all filters in the scans
        /// 3. Add each table to the from clause, including joins
        fn visit_child(plan: PlanNode) -> Result<(), String> {
            match plan {
                PlanNode::Aggregate { children, .. } => {
                    for child in children.unwrap() {
                        visit_child(child)?;
                    }
                }
                PlanNode::Gather { children, .. } => {
                    for child in children.unwrap() {
                        visit_child(child)?;
                    }
                }
                PlanNode::SeqScan {
                    parent_relationship,
                    relation_name,
                    alias,
                    filter,
                } => {}
                PlanNode::HashJoin {
                    parent_relationship,
                    join_type,
                    inner_unique,
                    hash_cond,
                    join_filter,
                    children,
                } => {}
                _ => {}
            }
            Ok(())
        }

        visit_child(plan);

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
            selection: None,
            cluster_by: vec![],
            connect_by: None,
            distribute_by: vec![],
            sort_by: sort_by,
            having: having,
            named_window: vec![],
            qualify: None,
            window_before_qualify: false,
            flavor: SelectFlavor::Standard,
            value_table_mode: None,
        };

        select
    }
}

trait FromStr {
    fn from_str(s: &str) -> Self;
}

impl FromStr for Expr {
    fn from_str(filter: &str) -> Self {
        let filter = filter.replace("(", "").replace(")", "");

        // Might need better split for more complex filters
        let filter = filter.split(" ").collect::<Vec<&str>>();

        let str_operator = filter[1];

        let operator = match str_operator {
            "=" => BinaryOperator::Eq,
            ">" => BinaryOperator::Gt,
            "<" => BinaryOperator::Lt,
            ">=" => BinaryOperator::GtEq,
            "<=" => BinaryOperator::LtEq,
            "!=" => BinaryOperator::NotEq,
            "AND" => BinaryOperator::And,
            "OR" => BinaryOperator::Or,
            "~~" => BinaryOperator::PGLikeMatch,
            "!~~" => BinaryOperator::PGNotILikeMatch,
            "+" => BinaryOperator::Plus,
            "-" => BinaryOperator::Minus,
            "*" => BinaryOperator::Multiply,
            "/" => BinaryOperator::Divide,
            "%" => BinaryOperator::Modulo,
            "~" => BinaryOperator::BitwiseXor,

            _ => panic!("Unsupported operator: {}", str_operator),
        };

        // match operator {
        //     BinaryOperator::Eq => {
        //         let left = Expr::BinaryOp(
        //             operator,
        //             Box::new(Expr::Identifier(filter[0].to_string())),
        //             Box::new(Expr::Identifier(filter[2].to_string())),
        //         );
        //         let right = Expr::Identifier(filter[4].to_string());
        //         Ok(Some(Expr::BinaryOp(
        //             operator,
        //             Box::new(left),
        //             Box::new(right),
        //         )))
        //     }
        //     _ => Err(format!("Unsupported operator: {}", str_operator)),
        // }

        // Ok(Some(filter))

        let expr = Expr::BinaryOp {
            op: operator,
            left: Box::new(Expr::Value(ValueWithSpan {
                value: Value::SingleQuotedString(filter[0].to_string()),
                span: Span::empty(),
            })),
            right: Box::new(Expr::Value(ValueWithSpan {
                value: Value::SingleQuotedString(filter[2].to_string()),
                span: Span::empty(),
            })),
        };

        expr
    }
}

// impl FromStr for Value {
//     fn from_str(value: &str) -> Result<Self, String> {
//         if value.starts_with("'") && value.ends_with("'") {
//             Ok(Value::String(value[1..value.len() - 1].to_string()))
//         } else {
//             Err(format!("Unsupported value: {}", value))
//         }
//     }
// }

pub fn test() {
    println!();
}
