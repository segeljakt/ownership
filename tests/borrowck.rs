use ownership_v4::ast::Function;

#[test]
fn check_function() {
    let f = Function::parse(
        "fn foo(): int =
             let mut x = 42 in
             let y = &mut x in
             assign(y.deref, 1);")
        .unwrap()
        .infer();
    println!("{}", f.verbose());
    let f = f.into_mir().compute_liveness();
    println!("{}", f.verbose());
    f.borrowck();
}
