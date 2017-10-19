type Ident = String;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    EInt(i64),
    EStr(String),
    EIdent(Ident),
    EList(Ident, Vec<Expr>),
}
