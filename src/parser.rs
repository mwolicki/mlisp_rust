use parser_combinators::*;
use expr::Expr;

pub fn parse<'a>(txt: &'a [char]) -> ParseResult<Expr> {
    let expr = refl_parser(|expr| {

        let expr_impl = vec![
            p_int().map(Expr::EInt),
            p_char('"').right(p_string().map(Expr::EStr)).left(
                p_char('"')
            ),

            p_string().map(Expr::EIdent),

            p_char('(')
                .right(spaces().right(p_string().left(spaces())))
                .both(all(spaces().right(expr).left(spaces())))
                .map(|(hd, tl)| Expr::EList(hd, tl))
                .left(p_char(')')),
        ];

        any(expr_impl)
    });

    expr.parse(txt)
}