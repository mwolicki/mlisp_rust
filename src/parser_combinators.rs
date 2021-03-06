
use std::marker::PhantomData;
use std;
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Debug)]
pub struct Corr<'a, T> {
    txt: &'a [char],
    pub res: T,
}

pub type ParseResult<'a, T> = Result<Corr<'a, T>, (&'a str, &'a [char])>;

pub fn p_char<'a>(ch: char) -> RcParser<'a, char> {
    LambdaParser::create(move |txt| if !txt.is_empty() && txt[0] == ch {
        Ok(Corr {
            txt: &txt[1..],
            res: ch,
        })
    } else {
        Err(("no char", txt))
    })
}

pub fn p_str<'a>(string: &'a str) -> RcParser<'a, &str> {
    let s: Vec<char> = string.chars().collect();
    LambdaParser::create(move |txt| if txt.starts_with(&s) {
        let corr = Corr {
            txt: &txt[s.len()..],
            res: string,
        };
        Ok(corr)
    } else {
        Err(("no str", txt))
    })
}

pub fn both<'a, A, ARet, B, BRet>(left: A, right: B) -> RcParser<'a, (ARet, BRet)>
where
    A: Parser<'a, Return = ARet> + 'a,
    B: Parser<'a, Return = BRet> + 'a,
    ARet: 'a,
    BRet: 'a,
{
    LambdaParser::create(move |txt| {
        let a_corr = left.parse(txt)?;
        let b_corr = right.parse(a_corr.txt)?;
        Ok(Corr {
            txt: b_corr.txt,
            res: (a_corr.res, b_corr.res),
        })
    })
}


pub fn left<'a, A, ARet, B, BRet>(left: A, right: B) -> RcParser<'a, ARet>
where
    A: Parser<'a, Return = ARet> + 'a,
    B: Parser<'a, Return = BRet> + 'a,
    ARet: 'a,
{
    LambdaParser::create(move |txt| {
        let a_corr = left.parse(txt)?;
        let b_corr = right.parse(a_corr.txt)?;
        Ok(Corr {
            txt: b_corr.txt,
            res: a_corr.res,
        })
    })
}


pub fn right<'a, A, ARet, B, BRet>(left: A, right: B) -> RcParser<'a, BRet>
where
    A: Parser<'a, Return = ARet> + 'a,
    B: Parser<'a, Return = BRet> + 'a,
    BRet: 'a,
{
    LambdaParser::create(move |txt| {
        let a_corr = left.parse(txt)?;
        let b_corr = right.parse(a_corr.txt)?;
        Ok(Corr {
            txt: b_corr.txt,
            res: b_corr.res,
        })
    })
}

pub fn map_parser<'a, Out, Fun, A, ARet>(parser: A, mapper: Fun) -> RcParser<'a, Out>
where
    Fun: Fn(ARet) -> Out + 'a,
    A: Parser<'a, Return = ARet> + 'a,
    Out: 'a,
{
    LambdaParser::create(move |txt| match parser.parse(txt) {

        Ok(cor) => {
            let f = &mapper;
            Ok(Corr {
                res: f(cor.res),
                txt: cor.txt,
            })
        }
        Err(e) => Err(e),
    })
}

pub fn all<'a, T>(parser: RcParser<'a, T>) -> RcParser<'a, Vec<T>>
where
    T: 'a,
{
    LambdaParser::create(move |txt| {
        let mut res = Vec::new();
        let mut txt = txt;
        while let Ok(corr) = parser.parse(txt) {
            res.push(corr.res);
            txt = corr.txt;
        }

        if res.is_empty() {
            Err(("all: no matches", txt))
        } else {
            Ok(Corr { res, txt })
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
            if let Ok(corr) = p.parse(txt) {
                return Ok(corr);
            }
        }

        Err(("any: no matches", txt))
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
            unreachable!()
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
    let c = Rc::clone(&x);
    let expr = scope(c);
    *x.as_ref().borrow_mut() = Some(expr);
    x
}

pub fn p_string<'a>() -> RcParser<'a, String> {
    let chars = ((b'*')..(b'z') + 1)
        .map(|x| p_char(x as char).as_rc())
        .collect::<Vec<_>>();

    all(any(chars)).map(|x| x.into_iter().collect::<String>())
}

pub fn p_int<'a>() -> RcParser<'a, i64> {
    let chars = ((b'0')..(b'9') + 1)
        .map(|x| p_char(x as char).as_rc())
        .collect::<Vec<_>>();

    all(any(chars)).map(|x| {
        x.into_iter().collect::<String>().parse::<i64>().unwrap()
    })
}

pub fn spaces<'a>() -> RcParser<'a, usize> {
    LambdaParser::create(|txt| {

        let c = txt.iter()
            .take_while(|x| **x == ' ' || **x == '\t' || **x == '\r' || **x == '\n')
            .count();

        Ok(Corr {
            res: c,
            txt: &txt[c..],
        })
    })
}

pub type RcParser<'a, R> = Rc<Parser<'a, Return = R> + 'a>;


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
        LambdaParser {
            phantom: PhantomData,
            f,
        }.as_rc()
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

pub trait Parser<'a> {
    type Return;
    fn parse(&self, txt: &'a [char]) -> ParseResult<'a, Self::Return>;

    fn as_rc(self) -> RcParser<'a, Self::Return>;

    fn both<B, BRet>(&self, right: B) -> RcParser<'a, (Self::Return, BRet)>
    where
        Self: Sized + 'a + Clone,
        B: Parser<'a, Return = BRet> + 'a,
        BRet: 'a,
    {
        both(self.clone(), right)
    }

    fn left<B, BRet>(&self, right: B) -> RcParser<'a, Self::Return>
    where
        B: Parser<'a, Return = BRet> + 'a,
        Self: Sized + 'a + Clone,
    {
        left(self.clone(), right)
    }

    fn map<Fun, Out>(&self, mapper: Fun) -> RcParser<'a, Out>
    where
        Fun: Fn(Self::Return) -> Out + 'a,
        Self: Sized + 'a + Clone,
        Out: 'a,
    {
        map_parser(self.clone(), mapper)
    }

    fn right<B, BRet>(&self, right_parser: B) -> RcParser<'a, BRet>
    where
        B: Parser<'a, Return = BRet> + 'a,
        Self: std::marker::Sized + 'a + Clone,
        BRet: 'a,
    {
        right(self.clone(), right_parser)
    }

    fn all(self) -> RcParser<'a, Vec<Self::Return>>
    where
        Self: Sized + 'a,
    {
        all(self.as_rc())
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
