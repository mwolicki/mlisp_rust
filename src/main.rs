#![feature(slice_patterns)]

mod parser_combinators;
mod parser;
mod expr;
mod eval;
use std::ffi::CStr;
use std::ffi::CString;
use std::os::raw::c_char;

fn main() {
    fn s(txt: &str) -> Result<expr::Expr, &str> {
        parser::parse(&txt.chars().collect::<Vec<char>>())
            .map(|x| eval::eval(&x.res).map(|(x,_)| x))
            .unwrap()
    }

    println!("{:?}", (s("(define sub1 (lambda (z) (- z 1))) (define sub2 (z) (- z 2))
(define or (a b) (if a true (if b true false)))

(define fib (a)
  (if (or (eq? a 1) (eq? a 2))
    1
    (+ (fib (sub1 a)) (fib (sub2 a)))))

(define downto (from f)
  (if (eq? 1 from) 
      (f from)
      (append  (f from) (downto (sub1 from) f))))

(downto 7 fib)")));
}


fn from_c_str(i: *mut c_char) -> String {
    unsafe { CStr::from_ptr(i).to_string_lossy().into_owned() }
}

fn to_c_str(s: &str) -> *mut c_char {
    CString::new(s)
        .expect("Couldn't convert to string.")
        .into_raw()
}

#[no_mangle]
pub fn js_run_code(code: *mut c_char) -> *mut c_char {
    let s = from_c_str(code).chars().collect::<Vec<char>>();
    let output = 
        parser::parse(&s).map(|x| eval::eval(&x.res).map(|(x,_)| format!("{}", x)))
        .unwrap_or_else(|_| Ok(String::from("parsing error.")))
        .unwrap_or_else(String::from);
    to_c_str(&output)
}
