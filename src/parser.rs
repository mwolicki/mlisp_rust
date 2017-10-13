use parser_combinators::*;

type Ident = String;

#[derive(Debug, Clone)]
pub enum Expr {
    EInt(i64),
    EStr(String),
    EIdent(Ident),
    EList(Vec<Expr>),
}

pub fn parse<'a>(txt: &'a [char]) -> ParseResult<Expr> {
    let expr = refl_parser(|expr| {
        let expr_impl = vec![
            p_int().map(|x| Expr::EInt(x)),
            parse_char('"')
                .right(p_string().map(|x| Expr::EStr(x)))
                .left(parse_char('"')),

            parse_char('"')
                .right(p_string().map(|x| Expr::EStr(x)))
                .left(parse_char('"')),

            p_string().map(|x| Expr::EIdent(x)),

            parse_char('(')
                .right(all(spaces().right(expr).left(spaces())))
                .map(|x| Expr::EList(x))
                .left(parse_char(')')),
        ];

        any(expr_impl)
    });

    all(expr).map(|x| Expr::EList(x)).parse(txt)
}