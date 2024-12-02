use ownership_v4::ast::Function;

#[test]
fn test_parse1() {
    Function::parse("fn f(mut x: i32, y: i32) -> i32 { 1 }").unwrap();
}

#[test]
fn test_parse2() {
    Function::parse("fn f(mut x: i32, y: i32) -> i32 { x.deref }").unwrap();
}

#[test]
fn test_parse3() {
    Function::parse("fn f(x: (i32, i32)) -> i32 { x.index(1) }").unwrap();
}

#[test]
fn test_parse4() {
    Function::parse("fn f(x: &{}mut i32) -> i32 { x.deref }").unwrap();
}

#[test]
fn test_parse5() {
    Function::parse("fn f(x: &{}mut i32) -> i32 { x }").unwrap();
}
