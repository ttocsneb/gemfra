#[test]
fn macro_tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/01-simple-route.rs");
    t.pass("tests/02-named-param.rs");
}
