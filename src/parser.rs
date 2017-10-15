use parser_combinators::*;
use expr::Expr;

pub fn parse<'a>(txt: &'a [char]) -> ParseResult<Expr> {
    let expr = refl_parser(|expr| {

        let expr_impl = vec![
            p_int().map(Expr::EInt),
            parse_char('"').right(p_string().map(Expr::EStr)).left(
                parse_char('"')
            ),

            parse_char('"').right(p_string().map(Expr::EStr)).left(
                parse_char('"')
            ),

            p_string().map(Expr::EIdent),

            parse_char('(')
                .right(spaces().right(p_string().left(spaces())))
                .both(all(spaces().right(expr).left(spaces())))
                .map(|(hd, tl)| Expr::EList(hd, tl))
                .left(parse_char(')')),
        ];

        any(expr_impl)
    });

    expr.parse(txt)
}