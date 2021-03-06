use std::collections::HashMap;
use expr::Expr;
use parser::parse;

type Name = String;
type Env = HashMap<Name, Expr>;
type ArgValue = Expr;
type Memoized = HashMap<(Name, Vec<ArgValue>), Expr>;

//fix signature to use &str
pub fn eval<'a, 'b>(exprs: &'b [Expr]) -> Result<(Expr, Env), &'a str> {

    fn eval<'a, 'b>(expr: &'b Expr, env: &mut Env, memoized:&mut Memoized) -> Result<Expr, &'a str> {
        match *expr {
            Expr::Symbol(_) | Expr::Fun(_,_) | Expr::QuotedList(_) | Expr::Bool(_) | Expr::Unit | Expr::Int(_) | Expr::Str(_) => Ok(expr.clone()),
            Expr::Ident(ref val) => {
                match val.as_str() {
                    "true" | "#t" => Ok(Expr::Bool(true)),
                    "false" | "#f" => Ok(Expr::Bool(false)),
                    "unit" => Ok(Expr::Unit),
                    _ if env.contains_key(val) => Ok(env[val].clone()),
                    _ => Ok(Expr::Ident(val.clone())),
                }
            }
            Expr::List(ref name, ref values) if name == "define" => {
                match *values.as_slice() {
                    [] => Err("cannot define <empty> of value <empty>"),
                    [Expr::Ident(ref name), ref val] => {
                        let definition = eval(val, env, memoized)?;
                        env.insert(name.to_owned(), definition);
                        Ok(Expr::Unit)
                    }
                    [Expr::Ident(ref name), Expr::List(ref first_arg, ref tail_args), ref func] => {
                        let mut args = tail_args.iter()
                            .map(|x| if let Expr::Ident(ref v) = *x {
                                Ok(v.to_owned())
                            } else {
                                Err("expected ident")
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        args.insert(0, first_arg.to_owned());
                        env.insert(name.to_owned(), Expr::Fun(args, Box::new(func.clone())));
                        Ok(Expr::Unit)
                    }
                    _ => Err("cannot define var/function"),
                }
            },
            Expr::List(ref name, ref values) if name == "lambda" => {
                match *values.as_slice() {
                    [Expr::List(ref first_arg, ref tail_args), ref func] => {
                        let mut args = tail_args.iter()
                            .map(|x| if let Expr::Ident(ref v) = *x {
                                Ok(v.to_owned())
                            } else {
                                Err("expected ident")
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        args.insert(0, first_arg.to_owned());
                        Ok(Expr::Fun(args, Box::new(func.clone())))
                    }
                    _ => Err("cannot define lambda"),
                }
            },
            Expr::List(ref name, ref values) if name == "if" => {
                match *values.as_slice() {
                    [ref pattern, ref lhs, ref rhs] => {
                        if eval(pattern, env, memoized)? == Expr::Bool(true) {
                            eval(lhs, env, memoized)
                        }
                        else{
                            eval(rhs, env, memoized)
                        }
                    }
                    _ => Err("wrongly defined if"),
                }
            },
            Expr::List(ref name, ref values) => {
                let vals = values
                    .iter()
                    .map(|v| eval(v, env, memoized))
                    .collect::<Result<Vec<_>, _>>()?;

                fn i64_calc<'a, F>(f: F, vals: &[Expr]) -> Result<Expr, &'a str>
                where
                    F: Fn(i64, i64) -> i64,
                {
                    let i = vals.iter()
                        .map(|x| if let Expr::Int(v) = *x {
                            Ok(v)
                        } else {
                            Err("expected int")
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    if let Some(init) = i.get(0) {
                        Ok(Expr::Int(i.iter().skip(1).fold(*init, |acc, &x| f(acc, x))))
                    }
                    else{
                        Err("expected at least one parameter.")
                    }
                };

                match name.as_str() {
                    "+" | "add" => i64_calc(|a, b| a + b, &vals),
                    "-" | "sub" => i64_calc(|a, b| a - b, &vals),
                    "/" | "div" => i64_calc(|a, b| a / b, &vals),
                    "*" | "mul" => i64_calc(|a, b| a * b, &vals),
                    "list" => Ok(Expr::QuotedList(vals)),
                    "append" => {
                        match *vals.as_slice(){
                            [ref v, Expr::QuotedList(ref xs)] =>
                                {
                                    let mut xs = xs.clone();
                                    xs.insert(0, v.clone());
                                    Ok(Expr::QuotedList(xs))
                                },
                            [ref lhs, ref rhs] =>
                                {
                                    Ok(Expr::QuotedList(vec!(lhs.clone(), rhs.clone())))
                                },
                            _ => Err("unsupported params for append")
                        }
                    },
                    "eq?" => {
                        if vals.is_empty() {
                            Ok(Expr::Bool(true))
                        }
                        else{
                            let first = vals[0].clone();
                            if vals.iter().all(|x| *x == first) {
                                Ok(Expr::Bool(true))
                            }
                            else{
                                Ok(Expr::Bool(false))
                            }

                        }
                    },
                    "quote" => {
                        if values.len() == 1{
                            if let Expr::List(ref id, ref vals) = values[0] {
                                let mut vals2 = vals.clone();
                                vals2.insert(0, Expr::Ident(id.clone()));
                                Ok(Expr::QuotedList(vals2))    
                            }
                            else{
                                Ok(values[0].clone())
                            }
                        }
                        else {
                            Ok(Expr::QuotedList(values.clone()))}
                        },
                    name if env.contains_key(name) =>{
                        let val = env[name].clone();
                        match val {
                            Expr::Fun(ref names, ref expr) if names.len() == vals.len() => {
                                let key = (name.to_owned(), vals.clone());
                                if let Some(val2) = memoized.clone().get(&key) {
                                    Ok(val2.clone())
                                }
                                else{
                                    let mut env = env.clone();

                                    for (n, val) in names.iter().zip(vals) {                     
                                        env.insert(n.to_owned(), val);
                                    }
                                    
                                    let x = eval(&expr.clone(), &mut env, memoized)?;
                                    let _ = memoized.entry(key).or_insert(x.clone());
                                    Ok(x)
                                }
                            },
                            _ => Ok(val)
                        }
                    },
                    _ => {
                        let mut atoms = Vec::with_capacity(vals.len());
                        atoms.push(Expr::Ident(name.to_owned()));
                        for i in vals {
                            atoms.push(i);
                        }
                        Ok(Expr::QuotedList(atoms))
                    }
                }
            }
        }
    }
    let mut env = HashMap::new();
    let mut memoized = HashMap::new();
    exprs.iter().fold(Ok(Expr::Unit), |_, expr| eval(expr, &mut env, &mut memoized)).map(|x| (x, env))
    
}

#[test]
fn eval_test() {
    fn s(txt: &str) -> Result<Expr, &str> {
        parse(&txt.chars().collect::<Vec<char>>())
            .map(|x| eval(&x.res).map(|(x,_)| x))
            .unwrap()
    }
    assert_eq!(s("(+ (* 2 2) 2 3 )"), Ok(Expr::Int(9)));
    assert_eq!(s("(list (list 5 6) 7)"), Ok(Expr::QuotedList(
            vec!(Expr::QuotedList(
                vec!(Expr::Int(5), Expr::Int(6))), Expr::Int(7)))));
    assert_eq!(s("(quote (+ 1 2))"), Ok(Expr::QuotedList(
            vec!(Expr::Ident(String::from("+")), Expr::Int(1), Expr::Int(2)))));
    assert_eq!(s("(define x 1)"), Ok(Expr::Unit));
    assert_eq!(s("(append 1 (quote (+ 3))))"), Ok(Expr::QuotedList(
        vec!(Expr::Int(1), Expr::Ident(String::from("+")), Expr::Int(3)))));
    assert_eq!(s("(if (eq? 1 1) 5 (6 7))"), Ok(Expr::Int(5)));
    assert_eq!(s("(if (eq? 1 2) 5 \"abc\")"), Ok(Expr::Str(String::from("abc"))));
    assert_eq!(s("(eq? 1 1 1)"), Ok(Expr::Bool(true)));
    assert_eq!(s("(eq? 1 2)"), Ok(Expr::Bool(false)));
    assert_eq!(s("(eq? (1 2) (1 2) (1 2))"), Ok(Expr::Bool(true)));
    assert_eq!(s("(eq? (1 2) (1 2) (1 1))"), Ok(Expr::Bool(false)));
    assert_eq!(s("(define x 'abc)
                  x"), Ok(Expr::Symbol(String::from("abc"))));
    assert_eq!(s("(define add2 (a) (+ a 2))"), Ok(Expr::Unit));
    assert_eq!(
        s(
            "(+           
                 (add 1 2 3)    1 2 (/ 1 2 3)    1 2)",
        ),
        Ok(Expr::Int(12))
    );

    assert_eq!(s("(define add2 (a) (+ a 2))
                  (define nine 9)
                  (add2 nine)"), Ok(Expr::Int(11)));

    assert_eq!(s("(define sub1 (z) (- z 1))
(define sub2 (z) (- z 2))
(define or (a b) (if a true (if b true false)))
(define fib (a)
(if (or (eq? a 1) (eq? a 2))
    1
    (+ (fib (sub1 a)) (fib (sub2 a)))))
(fib 8)"), Ok(Expr::Int(21)));


    assert_eq!(s("(define id (lambda (a) a)) (id 42)"), Ok(Expr::Int(42)));
}
