use std::marker::PhantomData;

#[derive(Debug)]
struct Corr<'a, T> {
    txt: &'a [char],
    res: T,
}

#[derive(Debug)]
enum ParseResult<'a, T> {
    Ok(Corr<'a, T>),
    Fail(&'a str, &'a [char]),
}

struct AndParser<'a, A, ARet, B, BRet>
where
    A: Parser<'a, ARet> + 'a,
    B: Parser<'a, BRet> + 'a,
{
    c: PhantomData<(ARet, BRet)>,
    a: &'a A,
    b: &'a B,
}

impl<'a, A, ARet, B, BRet> Parser<'a, (ARet, BRet)> for AndParser<'a, A, ARet, B, BRet>
where
    A: Parser<
        'a,
        ARet,
    >,
    B: Parser<
        'a,
        BRet,
    >,
{
    fn parse(&self, txt: &'a [char]) -> ParseResult<'a, (ARet, BRet)> {
        match self.a.parse(txt) {
            ParseResult::Ok(a_corr) => {
                match self.b.parse(a_corr.txt) {
                    ParseResult::Ok(b_corr) => ParseResult::Ok(Corr {
                        txt: b_corr.txt,
                        res: (a_corr.res, b_corr.res),
                    }),
                    ParseResult::Fail(res, txt) => ParseResult::Fail(res, txt),
                }
            }
            ParseResult::Fail(res, txt) => ParseResult::Fail(res, txt),
        }
    }
}

trait Parser<'a, Return> {
    fn parse(&self, txt: &'a [char]) -> ParseResult<'a, Return>;
}

fn and<'a, A, ARet, B, BRet>(a: &'a A, b: &'a B) -> AndParser<'a, A, ARet, B, BRet>
where
    A: Parser<'a, ARet> + 'a,
    B: Parser<'a, BRet> + 'a,
{
    AndParser {
        a,
        b,
        c: PhantomData,
    }
}

struct CharParser(char);

impl<'a> Parser<'a, char> for CharParser {
    fn parse(&self, txt: &'a [char]) -> ParseResult<'a, char> {
        if txt.len() > 0 && txt[0] == self.0 {
            ParseResult::Ok(Corr {
                txt: &txt[1..],
                res: self.0,
            })
        } else {
            ParseResult::Fail("no char", txt)
        }
    }
}

fn parse_char<'a>(ch: char) -> CharParser {
    CharParser(ch)
}



fn main() {
    let x = "abcd";
    let c: Vec<char> = x.chars().collect();
    let a = parse_char('a');
    let b = parse_char('b');

    let p = and(&a, &b);

    println!("Hello, world! {:?}", p.parse(&c));
    println!("Hello, world! {:?}", p.parse(&c));
}
