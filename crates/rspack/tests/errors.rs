use rspack::builder::Builder;
use rspack_core::{Compiler, Optimization};

macro_rules! assert_snapshot {
  ($compiler:expr) => {
    let compiler = $compiler;
    let error = compiler.unwrap_err();
    insta::assert_snapshot!(error);
  };
}

#[tokio::test(flavor = "multi_thread")]
async fn options() {
  assert_snapshot!(Compiler::builder()
    .optimization(Optimization::builder().module_ids("unknown".to_string()))
    .build());

  assert_snapshot!(Compiler::builder().target(vec![]).build());
}
