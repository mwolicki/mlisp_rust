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

#[derive(Clone, Copy)]
struct BothParser<'a, A, B>
where
    A: Parser<'a>,
    B: Parser<'a>,
{
    left: A,
    right: B,
    phantom: PhantomData<&'a i8>,
}

impl<'a, A, ARet, B, BRet> Parser<'a> for BothParser<'a, A, B>
where
    A: Parser<'a, Return = ARet>,
    A: Copy,
    B: Parser<'a, Return = BRet>,
    B: Copy,
{
    type Return = (ARet, BRet);
    fn parse(&self, txt: &'a [char]) -> ParseResult<'a, Self::Return> {
        match self.left.parse(txt) {
            ParseResult::Ok(a_corr) => {
                match self.right.parse(a_corr.txt) {
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


#[derive(Clone, Copy)]
struct LeftParser<'a, A, B>
where
    A: Parser<'a>,
    B: Parser<'a>,
{
    left: A,
    right: B,
    phantom: PhantomData<&'a i8>,
}

impl<'a, A, ARet, B> Parser<'a> for LeftParser<'a, A, B>
where
    A: Parser<'a, Return = ARet>,
    A: Copy,
    B: Parser<'a>,
    B: Copy,
{
    type Return = ARet;
    fn parse(&self, txt: &'a [char]) -> ParseResult<'a, Self::Return> {
        match self.left.parse(txt) {
            ParseResult::Ok(a_corr) => {
                match self.right.parse(a_corr.txt) {
                    ParseResult::Ok(b_corr) => ParseResult::Ok(Corr {
                        txt: b_corr.txt,
                        res: a_corr.res,
                    }),
                    ParseResult::Fail(res, txt) => ParseResult::Fail(res, txt),
                }
            }
            ParseResult::Fail(res, txt) => ParseResult::Fail(res, txt),
        }
    }
}



#[derive(Clone, Copy)]
struct RightParser<'a, A, B>
where
    A: Parser<'a>,
    B: Parser<'a>,
{
    left: A,
    right: B,
    phantom: PhantomData<&'a i8>,
}

impl<'a, A, B, RetB> Parser<'a> for RightParser<'a, A, B>
where
    A: Parser<'a>,
    A: Copy,
    B: Parser<'a, Return = RetB>,
    B: Copy,
{
    type Return = RetB;
    fn parse(&self, txt: &'a [char]) -> ParseResult<'a, Self::Return> {
        match self.left.parse(txt) {
            ParseResult::Ok(a_corr) => {
                match self.right.parse(a_corr.txt) {
                    ParseResult::Ok(b_corr) => ParseResult::Ok(Corr {
                        txt: b_corr.txt,
                        res: b_corr.res,
                    }),
                    ParseResult::Fail(res, txt) => ParseResult::Fail(res, txt),
                }
            }
            ParseResult::Fail(res, txt) => ParseResult::Fail(res, txt),
        }
    }
}

#[derive(Clone, Copy)]
struct AllParser<'a, P>
where
    P: Parser<'a> + 'a,
{
    parser: P,
    phantom: PhantomData<&'a PhantomData<P>>,
}


impl<'a, P, Ret> Parser<'a> for AllParser<'a, P>
where
    P: Parser<'a, Return = Ret>,
    Ret: 'a,
    P: Copy,
{
    type Return = Vec<Ret>;
    fn parse(&self, txt: &'a [char]) -> ParseResult<'a, Self::Return> {
        let mut res = Vec::new();
        let mut txt = txt;
        loop {
            if let ParseResult::Ok(corr) = self.parser.parse(txt) {
                res.push(corr.res);
                txt = corr.txt;
            } else {
                break;
            }
        }

        if res.len() > 0 {
            ParseResult::Ok(Corr { res, txt })
        } else {
            ParseResult::Fail("no matches", txt)
        }
    }
}


trait Parser<'a>
where
    Self: std::marker::Sized,
    Self: Copy,
{
    type Return;
    fn parse(&self, txt: &'a [char]) -> ParseResult<'a, Self::Return>;

    fn both<B, BRet>(self, right: B) -> BothParser<'a, Self, B>
    where
        B: Parser<'a, Return = BRet>,
        Self: std::marker::Sized,
    {
        BothParser {
            left: self,
            right,
            phantom: PhantomData,
        }
    }

    fn left<B, BRet>(self, right: B) -> LeftParser<'a, Self, B>
    where
        B: Parser<'a, Return = BRet>,
        Self: std::marker::Sized,
    {
        LeftParser {
            left: self,
            right,
            phantom: PhantomData,
        }
    }

    fn right<B, BRet>(self, right: B) -> RightParser<'a, Self, B>
    where
        B: Parser<'a, Return = BRet>,
        Self: std::marker::Sized,
    {
        RightParser {
            left: self,
            right,
            phantom: PhantomData,
        }
    }

    fn all(self) -> AllParser<'a, Self> {
        AllParser {
            parser: self,
            phantom: PhantomData,
        }
    }
}

#[derive(Clone, Copy)]
struct CharParser(char);

impl<'a> Parser<'a> for CharParser {
    type Return = char;
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
    let x = "abcddefg";
    let c: Vec<char> = x.chars().collect();


    let p = parse_char('a').both(parse_char('b')).left(parse_char('c'));



    println!("->: {:?}", p.parse(&c));
    println!("->: {:?}", p.right(parse_char('d')).parse(&c));
    println!("->: {:?}", p.right(parse_char('d').all()).parse(&c));
    println!("->: {:?}", parse_char('d').parse(&c));
}
