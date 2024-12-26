use ownership_v4::ast::Function;

#[test]
fn test_unused0() {
    Function::parse("fn f() -> i32 { let x = 1; let y = 2; y }")
        .unwrap()
        .infer()
        .into_mir()
        .inspect()
        .with_liveness()
        .with_remove_unused_variables()
        .inspect();
}
