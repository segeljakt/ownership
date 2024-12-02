use ownership_v4::ast::Function;

#[test]
fn test1() {
    let f = Function::parse("fn f(x: &{}int): int = x.deref;").unwrap().infer();
    println!("{}", f.verbose());
}

#[test]
fn test2() {
    let f = Function::parse("fn f(): int = 1;").unwrap().infer();
    println!("{}", f.verbose());
}

#[test]
fn test3() {
    let f = Function::parse("fn f(): int = let x = 1 in x;").unwrap().infer();
    println!("{}", f.verbose());
}


