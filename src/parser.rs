use parser_combinators::*;
use expr::Expr;

pub fn parse(txt: &[char]) -> ParseResult<Vec<Expr>> {
    let quote_mark = p_char('"');
    let expr = refl_parser(|expr| {
        let expr_impl = vec![
            p_int().map(Expr::Int),
            quote_mark.right(p_string().map(Expr::Str)).left(
                quote_mark
            ),

            p_char('\'').right(p_string()).map(Expr::Symbol),
            p_string().map(Expr::Ident),

            p_char('(')
                .right(spaces().right(p_string().left(spaces())))
                .both(all(spaces().right(expr).left(spaces())))
                .map(|(hd, tl)| Expr::List(hd, tl))
                .left(p_char(')')),

            p_char('(')
                .right(spaces().right(p_string().left(spaces())))
                .map(|hd| Expr::List(hd, Vec::new()))
                .left(p_char(')')),
        ];


        spaces().right(any(expr_impl).left(spaces()))
    });

    all(expr).parse(txt)
}