#[test]
fn tests() {
  let t = trybuild::TestCases::new();
  t.pass("tests/hook/named_struct.rs");
  t.compile_fail("tests/hook/unit_struct.rs");
  t.compile_fail("tests/hook/unnamed_struct.rs");
}
