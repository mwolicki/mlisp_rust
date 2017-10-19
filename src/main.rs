#![feature(slice_patterns)]

mod parser_combinators;
mod parser;
mod expr;
mod eval;
use std::ffi::CStr;
use std::ffi::CString;
use std::os::raw::c_char;

fn main() {

}


fn from_c_str(i: *mut c_char) -> String {
    unsafe { CStr::from_ptr(i).to_string_lossy().into_owned() }
}

fn to_c_str(s: &String) -> *mut c_char {
    CString::new(s.as_str())
        .expect("Couldn't convert to string.")
        .into_raw()
}

#[no_mangle]
pub fn js_run_code(code: *mut c_char) -> *mut c_char {
    let s = from_c_str(code).chars().collect::<Vec<char>>();
    let output = 
        parser::parse(&s).map(|x| eval::eval(&x.res).map(|(x,_)| format!("{}", x)))
        .unwrap_or_else(|_| Ok(String::from("parsing error.")))
        .unwrap_or_else(|e| String::from(e));
    to_c_str(&output)
}
