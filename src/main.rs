mod parser_combinators;
mod parser;

use parser_combinators::*;

fn main() {

    fn s(txt: &str) -> Vec<char> {
        txt.chars().collect::<Vec<char>>()
    }

    let c = s("abcddefg0");

    let p = parse_char('a').both(parse_char('b')).left(parse_char('c'));
    let p2 = parse_char('a').both(parse_char('b')).left(parse_char('c'));
    let p3 = parse_char('a').both(parse_char('b')).left(parse_char('c'));

    println!("1->: {:?}", p.parse(&c));
    println!("2->: {:?}", p.right(parse_char('d')).parse(&c));
    println!("3->: {:?}", p2.right(parse_char('d').all()).parse(&c));
    println!("4->: {:?}", parse_char('d').parse(&c));
    println!("5->: {:?}", parse_string("abcddefg").parse(&c));
    println!("6->: {:?}", parse_string("abc").right(p_string()).parse(&c));
    println!("7->: {:?}", p3.parse(&c).map(|_| 1));
    println!("8->: {:?}", parser::parse(&s("(01234567890)yz")));
}
