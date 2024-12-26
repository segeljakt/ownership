use ownership_v4::ast::Function;

fn check(s: &str) {
    let f = Function::parse(s).expect("Should parse").infer();
    println!("{}", f.verbose());
    let mut f = f.into_mir();
    f.compute_liveness();
    println!("{}", f.verbose());
    f.borrowck();
}

#[test]
fn test_places0() {
    check(
        r#"fn example() {
               let x = ("a", "b");
               let y = x.index(0);
               print(&y);
           }"#,
    );
}

#[test]
fn test_places1() {
    check(
        r#"fn example() {
               let x = ("a", ("b", "c"));
               let y = x.index(1);
               let z = y.index(1);
               print(&z);
           }"#,
    );
}

#[test]
fn test_immutable_borrow() {
    check(
        r#"fn example() {
               let x = "hello";
               let y = &x;
               print(y);
           }"#,
    );
}

#[test]
fn test_mutable_borrow() {
    check(
        r#"fn example() {
               let mut x = "hello";
               let y = &mut x;
               print(y);
           }"#,
    );
}

#[test]
fn test_mutable_borrow_deref_coerce() {
    check(
        r#"fn example() {
               let mut x = "hello";
               let y = &mut x;
               assign(y.deref, "world");
               print(y);
           }"#,
    );
}

#[test]
fn test_multiple_immutable_borrows() {
    check(
        r#"fn example() {
               let x = "hello";
               let a = &x;
               let b = &x;
               let c = &x;
               print(a);
               print(c);
               print(b);
           }"#,
    );
}

#[test]
fn test_copy_immutable_borrows() {
    check(
        r#"fn example() {
               let x = "hello";
               let a = &x;
               let b = a;
               let c = a;
               print(a);
               print(c);
               print(b);
           }"#,
    );
}

#[test]
fn test_move_mutable_borrow() {
    check(
        r#"fn example() {
               let x = "hello";
               let a = &x;
               let b = a;
               print(b);
           }"#,
    );
}

#[test]
#[should_panic]
fn test_err_borrow_conflict() {
    check(
        r#"fn example() {
               let mut x = "hello";
               let a = &x;
               let b = &mut x;
               print(a);
               print(b);
           }"#,
    );
}

#[test]
#[should_panic]
fn test_err_multiple_mutable_borrows() {
    check(
        r#"fn example() {
               let mut x = "hello";
               let a = &mut x;
               let b = &mut x;
               print(a);
               print(b);
           }"#,
    );
}

#[test]
#[should_panic]
fn test_err_move_immutable_deref() {
    check(
        r#"fn example() {
               let x = "hello";
               let a = &mut x;
               let b = a;
               let c = a;
               print(b);
           }"#,
    );
}

#[test]
#[should_panic]
fn test_err_move_mutable_deref() {
    check(
        r#"fn example() {
               let mut x = "hello";
               let a = &mut x;
               let b = a;
               print(a);
           }"#,
    );
}

#[test]
#[should_panic]
fn test_err_move_mutable() {
    check(
        r#"fn example() {
               let x = "hello";
               let a = &x;
               let b = a.deref;
               print(a);
           }"#,
    );
}

#[test]
fn test_reborrow() {
    check(
        r#"fn example() {
               let x = "hello";
               let a = &x;
               let b = &a.deref;
               print(a);
           }"#,
    );
}

#[test]
fn test_function0() {
    check(
        r#"fn foo(x: &String)) -> &'x String {
               x
           }"#,
    );
}

#[test]
fn test_function1() {
    check(r#"fn foo(x: &String, y: &String)) -> &{shared(x)} String { &x.deref }"#);
}

#[test]
fn test_function2() {
    check(r#"fn foo(xy: (&String, &String))) -> &{shared(xy.index(1)} String { xy.1 }"#);
}

#[test]
fn test_function3() {
    check(r#"fn foo(xy: (&String, &String))) -> &{shared(xy.index(2)} String { xy.1 }"#);
}

// // Example 6: Dangling Reference Prevention
// // This code is expected to fail due to a dangling reference, so we add #[should_panic].
// #[test]
// #[should_panic]
// fn test_6_dangling_reference_prevention() {
//     check(
//         r#"fn example5() {
//                let r;
//                {
//                    let x = 10;
//                    r = &x; // ERROR: `x` does not live long enough
//                }
//                print(r);
//            }"#,
//     );
// }
//
// // Example 7: Lifetime Inference in Functions
// // Demonstrates returning a reference safely with proper lifetimes.
// #[test]
// fn test_7_lifetime_inference() {
//     check(
//         r#"fn example6() {
//                fn return_ref<'a>(val: &'a i32) -> &'a i32 {
//                    val
//                }
//                let x = 5;
//                let y = return_ref(&x);
//                print(y);
//            }"#,
//     );
// }
//
// // Example 8: Structs With Lifetimes
// // Struct holding a reference must have an explicit lifetime.
// #[test]
// fn test_8_struct_with_lifetimes() {
//     check(
//         r#"fn example7() {
//                struct Wrapper<'a> {
//                    data: &'a i32,
//                }
//
//                let x = 10;
//                let w = Wrapper { data: &x };
//                print(w.data);
//            }"#,
//     );
// }
//
// // Example 9: Data Race Prevention (Illustrative)
// // If we tried to share `&mut x` across threads without sync, it wouldn't compile.
// // As is, this should compile, but changing it to actually move `a` into the thread
// // while also using `x` in main would fail.
// #[test]
// fn test_9_data_race_prevention() {
//     check(
//         r#"fn example8() {
//                let mut x = 0;
//                let a = &mut x;
//                // Imagine a scenario with concurrency: Rust won't let us misuse `a`.
//                *a = 42;
//                print(a);
//            }"#,
//     );
// }
//
// // Example 10: Non-Lexical Lifetimes
// // Demonstrates that once `r` is no longer used, we can re-borrow `x` as mutable.
// #[test]
// fn test_10_non_lexical_lifetimes() {
//     check(
//         r#"fn example9() {
//                let mut x = "hello";
//                let r = &x;
//                print(r); // last use of r here
//                // r ends now
//
//                let m = &mut x;
//                *m = "hello world";
//                print(m);
//            }"#,
//     );
// }
//
// // Example 11: Reborrowing References
// // Reborrow a mutable reference as an immutable one temporarily.
// #[test]
// fn test_11_reborrowing_references() {
//     check(
//         r#"fn example10() {
//                let mut val = 10;
//                {
//                    let m = &mut val;
//                    let i = &*m; // Reborrow mut as immut
//                    print(i);
//                    // `i` is not used after here
//                    *m += 1; // Safe now
//                }
//                print(val);
//            }"#,
//     );
// }
//
// // Example 12: Borrow Checker in Closures
// // Closure capturing a mutable reference ends its borrow when it goes out of scope.
// #[test]
// fn test_12_borrow_in_closures() {
//     check(
//         r#"fn example11() {
//                let mut num = 5;
//                {
//                    let mut add_num = |x: i32| num += x; // closure mutably borrows num
//                    add_num(10);
//                }
//                print(num);
//            }"#,
//     );
// }
//
// // Example 13: Interior Mutability with `RefCell`
// // Shows runtime borrow checking, still following compile-time reference rules.
// #[test]
// fn test_13_refcell_intermediate() {
//     check(
//         r#"fn example12() {
//                use std::cell::RefCell;
//                let x = RefCell::new(42);
//                {
//                    let mut_ref = x.borrow_mut();
//                    *mut_ref = 100;
//                }
//                let imm_ref_1 = x.borrow();
//                let imm_ref_2 = x.borrow();
//                print(*imm_ref_1);
//                print(*imm_ref_2);
//            }"#,
//     );
// }
//
// // Example 14: Explicit Lifetimes When Ambiguous
// // When returning references taken from multiple inputs, we need explicit lifetimes.
// #[test]
// fn test_14_explicit_lifetimes() {
//     check(
//         r#"fn example13() {
//                fn longest<'a>(a: &'a i32, b: &'a i32) -> &'a i32 {
//                    if a > b { a } else { b }
//                }
//
//                let x = 10;
//                let y = 20;
//                let r = longest(&x, &y);
//                print(r);
//            }"#,
//     );
// }
//
// // Example 15: Mutually Exclusive Borrows in Nested Scopes
// // Once the mutable borrow ends, we can immutably borrow again.
// #[test]
// fn test_15_mut_exclusive_in_nested_scopes() {
//     check(
//         r#"fn example14() {
//                let mut data = vec![1, 2, 3];
//                {
//                    let slice = &mut data[0..2];
//                    slice[0] = 10;
//                }
//                let first = &data[0];
//                print(first);
//            }"#,
//     );
// }
//
// // Example 16: Borrow Checking Across Function Boundaries
// // Uncommenting the call to `change` will fail the borrow checker, so we add #[should_panic].
// #[test]
// #[should_panic]
// fn test_16_borrow_across_function_boundaries() {
//     check(
//         r#"fn example15() {
//                fn change(value: &mut i32, other: &i32) {
//                    *value = *other + 1;
//                }
//
//                let mut x = 5;
//                let r = &x;
//                change(&mut x, r); // ERROR: can't mutably borrow `x` while `r` borrows it immutably
//                print(x);
//            }"#,
//     );
// }
//
// #[test]
// fn check_basic() {
//     let f = Function::parse(
//         r#"fn foo() -> String {
//              let x = "a";
//              let y = x;
//              let z = x;
//              z
//          }"#,
//     )
//     .unwrap()
//     .infer();
//     println!("{}", f);
//     let f = f.into_mir().compute_liveness();
//     println!("{}", f.verbose());
//     f.borrowck();
// }
//
// #[test]
// fn check_ref() {
//     let f = Function::parse(
//         r#"fn foo() -> String {
//              let x = "a";
//              let y = &x;
//              let z = &x;
//              z.deref
//          }"#,
//     )
//     .unwrap()
//     .infer();
//     println!("{}", f);
//     let f = f.into_mir().compute_liveness();
//     println!("{}", f);
//     println!("{}", f.verbose());
//     f.borrowck();
// }
//
// #[test]
// fn check_tuple() {
//     let f = Function::parse(
//         r#"fn foo() -> String {
//              let x = ("a", "b");
//              let y = x.index(1);
//              let z = x.index(1);
//              z
//          }"#,
//     )
//     .unwrap()
//     .infer();
//     println!("{}", f);
//     let f = f.into_mir().compute_liveness();
//     println!("{}", f);
//     f.borrowck();
// }
//
// #[test]
// fn check_function0() {
//     let f = Function::parse(
//         "fn foo() -> i32 {
//              let mut x = 42;
//              let y = &mut x;
//              assign(y.deref, 1);
//              y.deref
//          }",
//     )
//     .unwrap()
//     .infer();
//     println!("{}", f);
//     let f = f.into_mir().compute_liveness();
//     println!("{}", f);
//     f.borrowck();
// }
//
// #[test]
// fn check_function1() {
//     let f = Function::parse(
//         "fn foo() -> i32 {
//              let x = 42;
//              let y = &x;
//              y.deref
//          }",
//     )
//     .unwrap()
//     .infer();
//     println!("{}", f);
//     let f = f.into_mir().compute_liveness();
//     println!("{}", f.verbose());
//     f.borrowck();
// }
//
// #[test]
// fn check_function2() {
//     let f = Function::parse(
//         r#"fn foo() -> String {
//              let x = "hello";
//              let y = &x;
//              y.deref
//          }"#,
//     )
//     .unwrap()
//     .infer();
//     println!("{}", f);
//     let f = f.into_mir().compute_liveness();
//     println!("{}", f.verbose());
//     f.borrowck();
// }
