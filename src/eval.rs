use std::collections::HashMap;
use expr::Expr;

#[derive(Debug, Clone)]
pub enum Atom {
    Int(i64),
    String(String),
    Bool(bool),
    Unit,
}


type Name = String;
type Env = HashMap<Name, (Vec<Name>, Value)>;
type EnvBuildIn = HashMap<Name, (Vec<Name>, Box<Fn(Vec<Value>) -> Value>)>;

#[derive(Debug, Clone)]
pub enum Value {
    Atom(Atom),
    List(Box<Value>),
    Fun(Vec<Name>, Expr),
}

//fix signature to use &str
pub fn eval<'a, 'b>(expr: &'b Expr) -> Result<Value, &'a str> {

    fn eval<'a, 'b>(expr: &'b Expr, values: Env) -> Result<Value, &'a str> {
        match *expr {
            Expr::EInt(val) => Ok(Value::Atom(Atom::Int(val))),
            Expr::EStr(ref val) => Ok(Value::Atom(Atom::String(val.clone()))),
            Expr::EIdent(ref val) => {
                match val.as_str() {
                    "true" => Ok(Value::Atom(Atom::Bool(true))),
                    "false" => Ok(Value::Atom(Atom::Bool(false))),
                    "unit" => Ok(Value::Atom(Atom::Unit)),
                    _ => Err("expected true, false or unit."),
                }
            }
            Expr::EList(ref name, ref vals) => {
                let vals = vals.iter()
                    .map(|v| eval(&v, values.clone()))
                    .collect::<Result<Vec<_>, _>>()?;

                fn i64_calc<'a, F>(f: F, zero: i64, vals: &Vec<Value>) -> Result<Value, &'a str>
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
                    _ => Err("unexpected operation"),
                }
            }
        }
    }

    eval(&expr, HashMap::new())
}