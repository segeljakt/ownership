use ownership_v4::ast::Function;

#[test]
fn test1() {
    let f = Function::parse("fn f(x: i32) -> i32 { x }").unwrap().infer();
    println!("{}", f.verbose());
    println!("{}", f.into_mir().compute_liveness().verbose());
}

#[test]
fn test2() {
    let f = Function::parse("fn f(x: String) -> String { x }").unwrap().infer();
    println!("{}", f.verbose());
    println!("{}", f.into_mir().compute_liveness().verbose());
}
