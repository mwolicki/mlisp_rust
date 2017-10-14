enum Atom{
    Int(i64),
    String(String),
    Bool(bool)
}

#[derive(Debug)]
enum Value {
    Atom(Atom),
    List(Value),
    Function(Expr)
}


fn eval(expr: &Expr) {
    fn eval' (expr:&Expr, values)
}