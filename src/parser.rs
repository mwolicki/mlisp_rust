use parser_combinators::*;

type Ident = String;

#[derive(Debug, Clone, Copy)]
pub enum Expr<'a> {
    EInt(i64),
    EStr(&'a String),
    EIdent(&'a Ident),
    EList(&'a Ident, &'a [Expr<'a>]),
}


pub fn parse<'a>(txt: &'a [char]) -> ParseResult<Expr<'a>> {
    let expr = refl_parser(|expr| {
        let expr_impl = vec![
            p_int().map(|x| Expr::EInt(x)),
            parse_char('(').right(expr).left(parse_char(')')),
        ];

        any(expr_impl)
    });

    expr.parse(txt)
}