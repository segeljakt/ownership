use ownership_v4::ast::Function;

#[test]
fn test_dominator0() {
    let f = Function::parse("fn f() -> i32 { if true { 1 } else { 2 }; 3 }")
        .unwrap()
        .infer()
        .into_mir()
        .with_predecessors()
        .with_dominators();

    assert_eq!(f.domtree, vec![vec![1, 2, 3], vec![], vec![], vec![]]);
}

#[test]
fn test_dominator1() {
    let f = Function::parse("fn f() -> i32 { loop { break; } }")
        .unwrap()
        .infer()
        .into_mir()
        .with_predecessors()
        .with_dominators();

    assert_eq!(f.domtree, vec![vec![1, 2, 3], vec![], vec![], vec![]]);
}
