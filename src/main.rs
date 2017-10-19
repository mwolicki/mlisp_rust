#![feature(slice_patterns)]

mod parser_combinators;
mod parser;
mod expr;
mod eval;

use parser_combinators::*;

fn main() {

    fn s(txt: &str) -> Vec<char> {
        txt.chars().collect::<Vec<char>>()
    }

    let c = s("abcddefg0");

    let p = p_char('a').both(p_char('b')).left(p_char('c'));

    println!("1->: {:?}", p.parse(&c));
    println!("2->: {:?}", p.right(p_char('d')).parse(&c));
    println!("3->: {:?}", p.right(p_char('d').all()).parse(&c));
    println!("4->: {:?}", p_char('d').parse(&c));
    println!("5->: {:?}", p_str("abcddefg").parse(&c));
    println!("6->: {:?}", p_str("abc").right(p_string()).parse(&c));
    println!("7->: {:?}", p.parse(&c).map(|_| 1));
    println!(
        "8->: {:?}",
        parser::parse(&s(
            "(+           
                 (add 1 2 3)    1 2 (/ 1 2 3)    1 2)",
        )).map(|x| eval::eval(&x.res))
    );

    println!(
        "9->: {:?}",
        parser::parse(&s("(+ (* 2 2) 2 3 )")).map(|x| eval::eval(&x.res))
    );
}
