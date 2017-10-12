
use std::marker::PhantomData;
use std::*;
use std;
use std::rc::Rc;
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
        Fun: FnOnce(T) -> Out,
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



#[derive(Clone, Copy)]
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

#[derive(Clone, Copy)]
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


#[derive(Clone, Copy)]
pub struct LambdaParser<'a, Out, T>
where
    T: Fn(&'a [char]) -> ParseResult<'a, Out>,
{
    f: T,
    phantom: PhantomData<(&'a i8, Out)>,
}

pub fn parse_char(ch: char) -> CharParser {
    CharParser(ch)
}

#[derive(Clone, Copy)]
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


#[derive(Clone, Copy)]
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


#[derive(Clone, Copy)]
pub struct MapParser<'a, Out, Fun, A, ARet>
where
    Fun: FnOnce(ARet) -> Out,
    A: Parser<'a, Return = ARet>,
    Fun: Copy,
    ARet: Copy,
    Out: Copy,
{
    map: Fun,
    parser: A,
    phantom: PhantomData<(&'a i8, A, Fun)>,
}

impl<'a, Out, Fun, A, ARet> Parser<'a> for MapParser<'a, Out, Fun, A, ARet>
where
    Fun: FnOnce(ARet) -> Out
        + Copy
        + 'static,
    ARet: Copy + 'static,
    Out: Copy + 'static,
    A: Parser<'a, Return = ARet>
        + 'static,
{
    type Return = Out;
    fn parse(&self, txt: &'a [char]) -> ParseResult<'a, Out> {
        self.parser.parse(txt).map(self.map)
    }

    fn as_rc(self) -> RcParser<'a, Self::Return> {
        Rc::new(self)
    }
}

#[derive(Clone, Copy)]
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

#[derive(Clone, Copy)]
pub struct AllParser<'a, P>
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
        while let ParseResult::Ok(corr) = self.parser.parse(txt) {
            res.push(corr.res);
            txt = corr.txt;
        }

        if res.is_empty() {
            ParseResult::Fail("all: no matches", txt)
        } else {
            ParseResult::Ok(Corr { res, txt })
        }
    }

    fn as_rc(self) -> RcParser<'a, Self::Return> {
        Rc::new(self)
    }
}

#[derive(Clone, Copy)]
pub struct AnyParser<'a, P>
where
    P: Parser<'a> + 'a,
{
    parsers: &'a [P],
}

impl<'a, P, Ret> Parser<'a> for AnyParser<'a, P>
where
    P: Parser<'a, Return = Ret>,
    Ret: 'a,
    P: Copy,
{
    type Return = Ret;
    fn parse(&self, txt: &'a [char]) -> ParseResult<'a, Self::Return> {
        for p in self.parsers {
            if let ParseResult::Ok(corr) = p.parse(txt) {
                return ParseResult::Ok(corr);
            }
        }

        ParseResult::Fail("any: no matches", txt)
    }

    fn as_rc(self) -> RcParser<'a, Self::Return> {
        Rc::new(self)
    }
}

pub fn any<'a, P>(parsers: &'a [P]) -> AnyParser<'a, P>
where
    P: Parser<'a> + 'a,
{
    AnyParser { parsers }
}

pub struct ReflParser<'a, TRet, Par>(Option<Par>, PhantomData<(&'a i8, TRet)>)
where
    Par: Parser<'a, Return = TRet>,
    Par: Sized;


impl<'a, T, X> Parser<'a> for ReflParser<'a, T, X>
where
    X: Parser<'a, Return = T> + 'static + Sized,
    T: 'static,
    Self: Copy,
{
    type Return = T;
    fn parse(&self, txt: &'a [char]) -> ParseResult<'a, Self::Return> {
        if let Some(ref p) = self.0 {
            p.parse(txt)
        } else {
            panic!("implementation was not set");
        }
    }

    fn as_rc(self) -> RcParser<'a, Self::Return> {
        Rc::new(self)
    }
}

impl<'a, TRet, Par> ReflParser<'a, TRet, Par>
where
    Par: Parser<'a, Return = TRet>,
    Par: Sized,
{
    fn set(&mut self, p: Par) {
        self.0 = Some(p);
    }
}

pub fn refl_parser<'a, Ret, Par>() -> ReflParser<'a, Ret, Par>
where
    Par: Parser<'a, Return = Ret>,
    Par: Sized,
{
    ReflParser(None, PhantomData)
}

// pub fn p_string<'a>() -> AllParser<'a, AnyParser<'a, CharParser>> {
//     static mut PARSERS: Option<Vec<CharParser>> = None;
//     let p = unsafe {
//         if let None = PARSERS {
//             let chars = (('0' as u8)..('z' as u8))
//                 .map(|x| parse_char(x as char))
//                 .collect::<Vec<CharParser>>();

//             PARSERS = Some(chars);
//         }
//         if let Some(ref parser) = PARSERS {
//             any(parser)
//         } else {
//             panic!("blah")
//         }
//     };
//     p.all()
// }

// pub fn p_int<'a>() -> AllParser<'a, AnyParser<'a, CharParser>> {
//     static mut PARSERS: Option<Vec<CharParser>> = None;
//     let p = unsafe {
//         if let None = PARSERS {
//             let chars = (('0' as u8)..('9' as u8))
//                 .map(|x| parse_char(x as char))
//                 .collect::<Vec<CharParser>>();

//             PARSERS = Some(chars);
//         }
//         if let Some(ref parser) = PARSERS {
//             any(parser)
//         } else {
//             panic!("blah")
//         }
//     };
//     p.all()
// }

pub type RcParser<'a, R> = Rc<Parser<'a, Return = R> + 'a>;


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
        Self: Sized + Copy + 'a,
        Self::Return: Copy,
        BRet: Copy,
        B: Parser<'a, Return = BRet> + 'a + Copy,
    {
        as_rc(BothParser {
            left: self,
            right,
            phantom: PhantomData,
        })
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

    fn map<Fun, Out>(self, map: Fun) -> MapParser<'a, Out, Fun, Self, Self::Return>
    where
        Fun: FnOnce(Self::Return) -> Out,
        Fun: Copy,
        Out: Copy,
        Self::Return: Copy,
        Self: Copy,
        Self: std::marker::Sized,
    {
        MapParser {
            parser: self,
            map,
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

    fn all(self) -> RcParser<'a, Self::Return>
    where
        Self: Sized,
    {
        // (AllParser {
        //     parser: self,
        //     phantom: PhantomData,
        // }).as_rc()
        panic!("")

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
