use crate::postgres2plan::PlanNode;
use sqlparser::ast::helpers::attached_token::AttachedToken;
use sqlparser::ast::*;
/// Given a Postgres plan, convert it to a datafusion AST
///
///
///
///
pub fn plan2ast(_plan: PlanNode) -> Result<Query, String> {
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

pub fn test() {
    println!();
}
