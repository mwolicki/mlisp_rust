use std::collections::HashMap;
use expr::Expr;
use parser::parse;
use std::fmt;

#[derive(Clone, PartialEq)]
pub enum Atom {
    Int(i64),
    String(String),
    Ident(String),
    Bool(bool),
    Unit,
}

impl fmt::Display for Atom {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Atom::Int(i) => write!(f, "{}", i),
            &Atom::String(ref i) => write!(f, "\"{}\"", i),
            &Atom::Ident(ref i) => write!(f, "{}", i),
            &Atom::Bool(i) => write!(f, "{}", i),
            &Atom::Unit => write!(f, "unit"),
        }
    }
}

type ArgName = Name;

type Name = String;
type Env = HashMap<Name, Value>;

#[derive(Clone, PartialEq)]
pub enum Value {
    Atom(Atom),
    List(Box<Vec<Value>>), 
    Fun(Vec<ArgName>, Expr),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Value::Atom(ref i) => write!(f, "{}", i),
            &Value::List(ref items) => {
                write!(f, "(")?;
                for i in items.iter() {
                    write!(f, " {} ", i)?;
                }
                write!(f, ")")},
            &Value::Fun(_, _) => write!(f, "##fun##"),
        }
    }
}



//fix signature to use &str
pub fn eval<'a, 'b>(exprs: &'b Vec<Expr>) -> Result<(Value, Env), &'a str> {

    fn eval<'a, 'b>(expr: &'b Expr, env: &mut Env) -> Result<Value, &'a str> {
        match *expr {
            Expr::EInt(val) => Ok(Value::Atom(Atom::Int(val))),
            Expr::EStr(ref val) => Ok(Value::Atom(Atom::String(val.clone()))),
            Expr::EIdent(ref val) => {
                match val.as_str() {
                    "true" | "#t" => Ok(Value::Atom(Atom::Bool(true))),
                    "false" | "#f" => Ok(Value::Atom(Atom::Bool(false))),
                    "unit" => Ok(Value::Atom(Atom::Unit)),
                    _ if env.contains_key(val) => Ok(env[val].clone()),
                    _ => Ok(Value::Atom(Atom::Ident(val.clone()))),
                }
            }
            Expr::EList(ref name, ref values) if name == "define" => {
                match values.as_slice() {
                    &[] => Err("cannot define <empty> of value <empty>"),
                    &[Expr::EIdent(ref name), ref val] => {
                        let definition = eval(val, env)?;
                        env.insert(name.to_owned(), definition);
                        Ok(Value::Atom(Atom::Unit))
                    }
                    &[Expr::EIdent(ref name), Expr::EList(ref first_arg, ref tail_args), ref func] => {
                        let mut args = tail_args.iter()
                            .map(|x| if let Expr::EIdent(ref v) = *x {
                                Ok(v.to_owned())
                            } else {
                                Err("expected int")
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        args.insert(0, first_arg.to_owned());
                        env.insert(name.to_owned(), Value::Fun(args, func.clone()));
                        Ok(Value::Atom(Atom::Unit))
                    }
                    _ => Err("cannot define var/function"),
                }
            },
            Expr::EList(ref name, ref values) => {
                let vals = values
                    .iter()
                    .map(|v| eval(v, env))
                    .collect::<Result<Vec<_>, _>>()?;

                fn i64_calc<'a, F>(f: F, zero: i64, vals: &[Value]) -> Result<Value, &'a str>
                where
                    F: Fn(i64, i64) -> i64,
                {
                    let i = vals.iter()
                        .map(|x| if let Value::Atom(Atom::Int(v)) = *x {
                            Ok(v)
                        } else {
                            Err("expected int")
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    Ok(Value::Atom(
                        Atom::Int(i.iter().fold(zero, |acc, &x| f(acc, x))),
                    ))
                };

                match name.as_str() {
                    "+" | "add" => i64_calc(|a, b| a + b, 0, &vals),
                    "-" | "sub" => i64_calc(|a, b| a - b, 0, &vals),
                    "/" | "div" => i64_calc(|a, b| a / b, 1, &vals),
                    "*" | "mul" => i64_calc(|a, b| a * b, 1, &vals),
                    "list" => Ok(Value::List(Box::new(vals))),
                    name if env.contains_key(name) =>{
                        let val = env[name].clone();
                        match val {
                            Value::Atom(_) => Ok(val),
                            Value::List(_) => Ok(val),
                            Value::Fun(ref names, ref expr) if names.len() == vals.len() => {
                                let mut env = env.clone();

                                for (name, val) in names.iter().zip(vals) {
                                    env.insert(name.to_owned(), val);
                                }

                                eval(&expr.clone(), &mut env)
                            },
                            _ => Err("undefinde var")
                        }
                    },
                    _ => {
                        let mut atoms = Vec::with_capacity(vals.len());
                        atoms.push(Value::Atom(Atom::Ident(name.to_owned())));
                        for i in vals {
                            atoms.push(i);
                        }
                        Ok(Value::List(Box::new(atoms)))
                    }
                }
            }
        }
    }
    let mut env = HashMap::new();
    exprs.iter().fold(Ok(Value::Atom(Atom::Unit)), |_, expr| eval(expr, &mut env)).map(|x| (x, env))
    
}


fn s<'a>(txt: &'a str) -> Result<Value, &'a str> {
    parse(&txt.chars().collect::<Vec<char>>())
        .map(|x| eval(&x.res).map(|(x,_)| x))
        .unwrap()
}

#[test]
fn eval_test() {
    assert_eq!(s("(+ (* 2 2) 2 3 )"), Ok(Value::Atom(Atom::Int(9))));
    assert_eq!(s("(define x 1)"), Ok(Value::Atom(Atom::Unit)));
    assert_eq!(s("(define add2 (a) (+ a 2))"), Ok(Value::Atom(Atom::Unit)));
    assert_eq!(
        s(
            "(+           
                 (add 1 2 3)    1 2 (/ 1 2 3)    1 2)",
        ),
        Ok(Value::Atom(Atom::Int(12)))
    );

    assert_eq!(s("(define add2 (a) (+ a 2))
                  (define nine 9)
                  (add2 nine)"), Ok(Value::Atom(Atom::Int(11))));
    
}
