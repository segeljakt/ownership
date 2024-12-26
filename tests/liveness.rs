use ownership_v4::ast::Function;

#[test]
fn test() {
    let f = Function::parse("fn f() -> i32 { let x = 1; x }").unwrap().infer();
    println!("{f}");
    let f = f.into_mir().with_liveness();
    println!("{f}");
}
