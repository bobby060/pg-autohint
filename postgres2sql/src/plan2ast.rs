use crate::postgres2plan::*;
use sqlparser::ast::helpers::attached_token::AttachedToken;
use sqlparser::ast::*;
use sqlparser::tokenizer::Span;
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
    let expr = plan.visit_plan_node()?;

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
        /// Conceptuallly, will need to perform the following:
        /// 1. Extract the final projection
        /// 2. Create an expression tree for all filters in the scans
        /// 3. Add each table to the from clause, including joins
        ///
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
        let projection: Vec<SelectItem> = vec![];
        let mut from: Vec<TableWithJoins> = vec![];
        let mut group_by: GroupByExpr = GroupByExpr::All(vec![]);
        let mut sort_by: Vec<Expr> = vec![];
        let mut having: Option<Expr> = None;

        if let Some(filter) = self.filter {
            let filter = FromStr::from_str(&filter)?;
            having = Some(filter);
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

        // Ok(SetExpr::Select(Box::new(select)))

        Err("Not implemented".to_string())
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

trait FromStr: Sized {
    fn from_str(s: &str) -> Result<Self, String>;
}

impl FromStr for Expr {
    fn from_str(filter: &str) -> Result<Self, String> {
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

        Ok(expr)
    }
}

impl FromStr for Value {
    fn from_str(value: &str) -> Result<Self, String> {
        if value.starts_with("'") && value.ends_with("'") {
            Ok(Value::SingleQuotedString(
                value[1..value.len() - 1].to_string(),
            ))
        } else {
            Err(format!("Unsupported value: {}", value))
        }
    }
}

impl FromStr for ObjectName {
    fn from_str(name: &str) -> Result<Self, String> {
        Ok(ObjectName::from(vec![Ident {
            value: name.to_string(),
            quote_style: None,
            span: Span::empty(),
        }]))
    }
}

impl FromStr for Ident {
    fn from_str(name: &str) -> Result<Self, String> {
        Ok(Ident {
            value: name.to_string(),
            quote_style: None,
            span: Span::empty(),
        })
    }
}
pub fn test() {
    println!();
}
