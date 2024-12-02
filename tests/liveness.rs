use ownership_v4::ast::Function;

#[test]
fn test() {
    let f = Function::parse("fn f(): int = let x = 1 in x;")
        .unwrap()
        .into_mir()
        .compute_liveness();

    println!("{}", f);
}
