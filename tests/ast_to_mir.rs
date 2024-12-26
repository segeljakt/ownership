use ownership_v4::ast::Function;

macro_rules! check {
    ($a:expr, $b:expr) => {
        assert!($a == $b, "\n{}\n{}", $b, $a);
    };
}

#[test]
fn test1() {
    let f = Function::parse("fn f(x: i32) -> i32 { x }")
        .unwrap()
        .infer()
        .into_mir();
    check!(
        f.to_string(),
        indoc::indoc! {
        "fn f(x: i32): i32 {
             let _0: i32;
             bb0: {
                 _0 = copy x;
                 return;
             }
         }"}
    );
}

#[test]
fn test2() {
    let f = Function::parse("fn f(x: String) -> String { x }")
        .unwrap()
        .infer()
        .into_mir();
    check!(
        f.to_string(),
        indoc::indoc! {
        "fn f(x: String): String {
             let _0: String;
             bb0: {
                 _0 = move x;
                 return;
             }
         }"}
    );
}

#[test]
fn test3() {
    let f = Function::parse("fn f(x: bool) { loop { if x { break } else { continue } } }")
        .unwrap()
        .infer()
        .into_mir()
        .with_predecessors()
        .with_successors()
        .with_merge_blocks()
        .with_remove_unreachable()
        .with_remove_unused_variables()
        .with_predecessors()
        .with_successors()
        .with_dominators()
        .with_postorder()
        .with_reverse_postorder_number()
        .into_ast();
    check!(
        f.to_string(),
        indoc::indoc! {
        ""}
    );
}
