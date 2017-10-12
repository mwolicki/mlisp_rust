use ::*;
type Ident = String;

#[derive(Debug, Clone, Copy)]
pub enum Expr<'a> {
    EInt(i64),
    EStr(&'a String),
    EIdent(&'a Ident),
    EList(&'a Ident, &'a [Expr<'a>]),
}


pub fn parse() -> Expr<'static> {
    // let mut expr = refl_parser();

    // let exprImpl = vec![parse_char('x').map(|_| Expr::EInt(1))];
    // let x = any(&exprImpl);
    // expr.set(x);

    Expr::EInt(1)

}