
use std::marker::PhantomData;
use std;
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Debug)]
pub struct Corr<'a, T> {
    txt: &'a [char],
    res: T,
}

#[derive(Debug)]
pub enum ParseResult<'a, T> {
    Ok(Corr<'a, T>),
    Fail(&'a str, &'a [char]),
}

impl<'a, T> ParseResult<'a, T> {
    pub fn map<Fun, Out>(self, mapper: Fun) -> ParseResult<'a, Out>
    where
        Fun: Fn(T) -> Out,
    {
        match self {
            ParseResult::Ok(corr) => ParseResult::Ok(Corr {
                txt: corr.txt,
                res: mapper(corr.res),
            }),
            ParseResult::Fail(reason, txt) => ParseResult::Fail(reason, txt),
        }
    }
}



#[derive(Debug)]
pub struct CharParser(char);

impl<'a> Parser<'a> for CharParser {
    type Return = char;
    fn parse(&self, txt: &'a [char]) -> ParseResult<'a, char> {
        if !txt.is_empty() && txt[0] == self.0 {
            ParseResult::Ok(Corr {
                txt: &txt[1..],
                res: self.0,
            })
        } else {
            ParseResult::Fail("no char", txt)
        }
    }
    fn as_rc(self) -> RcParser<'a, Self::Return> {
        Rc::new(self)
    }
}

pub struct StringParser<'a> {
    txt: &'a str,
}

pub fn parse_string<'a>(s: &'a str) -> StringParser<'a> {
    StringParser { txt: s }
}

impl<'a> Parser<'a> for StringParser<'a> {
    type Return = &'a str;
    fn parse(&self, txt: &'a [char]) -> ParseResult<'a, &'a str> {
        let s: Vec<char> = self.txt.chars().collect();
        if txt.starts_with(&s) {
            let corr = Corr {
                txt: &txt[s.len()..],
                res: self.txt,
            };
            ParseResult::Ok(corr)
        } else {
            ParseResult::Fail("no char", txt)
        }
    }

    fn as_rc(self) -> RcParser<'a, Self::Return> {
        Rc::new(self)
    }
}



pub fn parse_char(ch: char) -> CharParser {
    CharParser(ch)
}

pub struct BothParser<'a, A, B>
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
    A: Parser<'a, Return = ARet> + 'a,
    B: Parser<'a, Return = BRet> + 'a,
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

    fn as_rc(self) -> RcParser<'a, Self::Return> {
        Rc::new(self)
    }
}


pub struct LeftParser<'a, A, B>
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
    A: Parser<'a, Return = ARet> + 'a,
    B: Parser<'a> + 'a,
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

    fn as_rc(self) -> RcParser<'a, Self::Return> {
        Rc::new(self)
    }
}


pub struct MapParser<'a, Out, Fun, A, ARet>
where
    Fun: FnOnce(ARet) -> Out,
    A: Parser<'a, Return = ARet>,
{
    map: Fun,
    parser: A,
    phantom: PhantomData<(&'a i8, A, Fun)>,
}

impl<'a, Out, Fun, A, ARet> Parser<'a> for MapParser<'a, Out, Fun, A, ARet>
where
    Fun: Fn(ARet) -> Out + 'a,
    A: Parser<'a, Return = ARet>
        + 'a,
    ARet: 'a,
    Out: 'a,
{
    type Return = Out;
    fn parse(&self, txt: &'a [char]) -> ParseResult<'a, Out> {
        self.parser.parse(txt).map(&self.map)
    }

    fn as_rc(self) -> RcParser<'a, Self::Return> {
        Rc::new(self)
    }
}

pub struct RightParser<'a, A, B>
where
    A: Parser<'a> + 'a,
    B: Parser<'a> + 'a,
{
    left: A,
    right: B,
    phantom: PhantomData<&'a i8>,
}

impl<'a, A, B, RetB> Parser<'a> for RightParser<'a, A, B>
where
    A: Parser<'a> + 'a,
    B: Parser<'a, Return = RetB> + 'a,
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

    fn as_rc(self) -> RcParser<'a, Self::Return> {
        Rc::new(self)
    }
}

pub fn all<'a, T>(parser: RcParser<'a, T>) -> RcParser<'a, Vec<T>>
where
    T: 'a,
{
    LambdaParser::create(move |txt| {
        let mut res = Vec::new();
        let mut txt = txt;
        while let ParseResult::Ok(corr) = parser.parse(txt) {
            res.push(corr.res);
            txt = corr.txt;
        }

        if res.is_empty() {
            ParseResult::Fail("all: no matches", txt)
        } else {
            ParseResult::Ok(Corr { res, txt })
        }
    })
}

pub fn any<'a, T>(parsers: Vec<RcParser<'a, T>>) -> RcParser<'a, T>
where
    T: 'a,
{
    let ps = Rc::new(parsers);
    LambdaParser::create(move |txt| {
        let parsers = ps.as_ref();
        for p in parsers {
            if let ParseResult::Ok(corr) = p.parse(txt) {
                return ParseResult::Ok(corr);
            }
        }

        ParseResult::Fail("any: no matches", txt)
    })
}

impl<'a, TRet> Parser<'a> for RefCell<Option<RcParser<'a, TRet>>>
where
    TRet: 'a,
{
    type Return = TRet;
    fn parse(&self, txt: &'a [char]) -> ParseResult<'a, Self::Return> {
        let x = self.borrow();
        if let Some(ref p) = *x {
            p.parse(txt)
        } else {
            panic!("implementation was not set");
        }
    }

    fn as_rc(self) -> RcParser<'a, Self::Return> {
        Rc::new(self)
    }
}


pub fn refl_parser<'a, Ret, S>(scope: S) -> RcParser<'a, Ret>
where
    S: FnOnce(RcParser<'a, Ret>) -> RcParser<'a, Ret> + 'a,
    Ret: 'a,
{
    let x: Rc<_> = Rc::new(RefCell::new(None));
    let expr = scope(x.clone());
    *x.as_ref().borrow_mut() = Some(expr);
    x
}

pub fn p_string<'a>() -> RcParser<'a, String> {
    let chars = (('0' as u8)..('z' as u8))
        .map(|x| parse_char(x as char).as_rc())
        .collect::<Vec<_>>();

    any(chars).all().map(|x| x.into_iter().collect::<String>())
}

pub fn p_int<'a>() -> RcParser<'a, i64> {
    let chars = (('0' as u8)..('9' as u8))
        .map(|x| parse_char(x as char).as_rc())
        .collect::<Vec<_>>();
    //print!("{}", chars);
    any(chars).all().map(|x| {
        x.into_iter().collect::<String>().parse::<i64>().unwrap()
    })
}

pub type RcParser<'a, R> = Rc<Parser<'a, Return = R> + 'a>;


#[derive(Clone, Copy)]
pub struct LambdaParser<'a, Out, T>
where
    T: Fn(&'a [char]) -> ParseResult<'a, Out>,
{
    f: T,
    phantom: PhantomData<(&'a i8, Out)>,
}

impl<'a, Out, T> LambdaParser<'a, Out, T>
where
    T: Fn(&'a [char]) -> ParseResult<'a, Out> + 'a,
    Out: 'a,
{
    fn create(f: T) -> RcParser<'a, Out> {
        as_rc(LambdaParser {
            phantom: PhantomData,
            f,
        })
    }
}

impl<'a, Out, T> Parser<'a> for LambdaParser<'a, Out, T>
where
    T: Fn(&'a [char]) -> ParseResult<'a, Out> + 'a,
    Out:'a
{
    type Return = Out;

    fn parse(&self, txt: &'a [char]) -> ParseResult<'a, Self::Return> {
        let f = &self.f;
        f(txt)
    }

    fn as_rc(self) -> RcParser<'a, Self::Return> {
        Rc::new(self)
    }
}


fn as_rc<'a, P, R>(p: P) -> RcParser<'a, R>
where
    P: Parser<'a, Return = R> + 'a,
{
    p.as_rc()
}

pub trait Parser<'a> {
    type Return;
    fn parse(&self, txt: &'a [char]) -> ParseResult<'a, Self::Return>;

    fn as_rc(self) -> RcParser<'a, Self::Return>;

    fn both<B, BRet>(self, right: B) -> RcParser<'a, (Self::Return, BRet)>
    where
        Self: Sized + 'a,
        B: Parser<'a, Return = BRet> + 'a,
    {
        as_rc(BothParser {
            left: self,
            right,
            phantom: PhantomData,
        })
    }

    fn left<B, BRet>(self, right: B) -> RcParser<'a, Self::Return>
    where
        B: Parser<'a, Return = BRet> + 'a,
        Self: Sized + 'a,
    {
        as_rc(LeftParser {
            left: self,
            right,
            phantom: PhantomData,
        })

    }

    fn map<Fun, Out>(self, map: Fun) -> RcParser<'a, Out>
    where
        Fun: Fn(Self::Return) -> Out + 'a,
        Self: Sized + 'a,
        Out: 'a,
    {
        as_rc(MapParser {
            parser: self,
            map,
            phantom: PhantomData,
        })
    }

    fn right<B, BRet>(self, right: B) -> RcParser<'a, BRet>
    where
        B: Parser<'a, Return = BRet> + 'a,
        Self: std::marker::Sized + 'a,
    {
        as_rc(RightParser {
            left: self,
            right,
            phantom: PhantomData,
        })
    }

    fn all(self) -> RcParser<'a, Vec<Self::Return>>
    where
        Self: Sized + 'a,
    {
        all(as_rc(self))

    }
}

impl<'a, R> Parser<'a> for RcParser<'a, R> {
    type Return = R;
    fn parse(&self, txt: &'a [char]) -> ParseResult<'a, Self::Return> {
        let x = self.as_ref();
        x.parse(txt)
    }

    fn as_rc(self) -> RcParser<'a, Self::Return> {
        self
    }
}
