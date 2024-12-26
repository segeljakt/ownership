use ownership_v4::ast::Function;

#[test]
fn test1() {
    let f = Function::parse("fn f(x: &{}i32) -> i32 { x.deref }")
        .unwrap()
        .infer();
    println!("{}", f.verbose());
}

#[test]
fn test2() {
    let f = Function::parse("fn f() -> i32 { 1 }").unwrap().infer();
    println!("{}", f.verbose());
}

#[test]
fn test3() {
    let f = Function::parse("fn f() -> i32 { let x = 1; x }")
        .unwrap()
        .infer();
    println!("{}", f.verbose());
}
