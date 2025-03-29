use crate::hints::PgHint;

/// optimize by replacing the input join method with another
pub fn optimize_join_method(hint: PgHint) -> PgHint {
    hint
    // TODO: implement optimize logic
}

/// optimize by replacing the input access method with another
pub fn optimize_access_method(hint: PgHint) -> PgHint {
    hint
    // TODO: implement optimize logic
}
