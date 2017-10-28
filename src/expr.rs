use std::fmt;

type Ident = String;
type ArgName = Ident;

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum Expr {
    Int(i64),
    Str(String),
    Bool(bool),
    Symbol(String),
    Unit,
    Ident(Ident),
    List(Ident, Vec<Expr>),
    QuotedList(Vec<Expr>),
    Fun(Vec<ArgName>, Box<Expr>),
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Expr::Int(i) => write!(f, "{}", i),
            Expr::Str(ref i) => write!(f, "\"{}\"", i),
            Expr::Symbol(ref i) => write!(f, "{}", i),
            Expr::Ident(ref i) => write!(f, "{}", i),
            Expr::Bool(i) => write!(f, "{}", i),
            Expr::Unit => write!(f, "unit"),
            Expr::List(ref ident, ref items) => {
                write!(f, "(")?;
                write!(f, " {} ", ident)?;
                for i in items.iter() {
                    write!(f, " {} ", i)?;
                }
                write!(f, ")")},
            Expr::QuotedList(ref items) => {
                write!(f, "(")?;
                for i in items.iter() {
                    write!(f, " {} ", i)?;
                }
                write!(f, ")")},
            Expr::Fun(_, _) => write!(f, "##fun##"),
        }
    }
}
