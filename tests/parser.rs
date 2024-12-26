use ownership_v4::ast::Function;

#[test]
fn test_parse1() {
    let f = Function::parse("fn f(mut x: i32, y: i32) -> i32 { 1 }").unwrap();
    println!("{f}");
}

#[test]
fn test_parse2() {
    let f = Function::parse("fn f(mut x: i32, y: i32) -> i32 { x.deref }").unwrap();
    println!("{f}");
}

#[test]
fn test_parse3() {
    let f = Function::parse("fn f(x: (i32, i32)) -> i32 { x.index(1) }").unwrap();
    println!("{f}");
}

#[test]
fn test_parse4() {
    let f = Function::parse("fn f(x: &{}mut i32) -> i32 { x.deref }").unwrap();
    println!("{f}");
}

#[test]
fn test_parse5() {
    let f = Function::parse("fn f(x: &{}mut i32) -> i32 { x }").unwrap();
    println!("{f}");
}

#[test]
fn test_parse6() {
    let f = Function::parse("fn f() { loop { break; continue; } }").unwrap();
    println!("{f}");
}

#[test]
fn test_parse7() {
    let f = Function::parse("fn f() { loop { loop { loop { } } } }").unwrap();
    println!("{f}");
}
