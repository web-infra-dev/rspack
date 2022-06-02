use crate::common::compile_fixture;

#[tokio::test]
async fn constant_folding() {
  let bundler = compile_fixture("constant-folding").await;
  let code = bundler
    .bundle
    .context
    .assets
    .lock()
    .unwrap()
    .get(0)
    .expect("failed to generate bundle")
    .source
    .to_owned();

  assert!(!code.contains("111"));
  assert!(code.contains("222"));

  assert!(!code.contains("333"));
  assert!(code.contains("444"));

  // FIXME: these tests are easily broken. Maybe we could find a better way to do it?
  // let sm = get_inline_source_map(&code);
  // let token1 = sm.lookup_token(207, 0).unwrap();
  // let token2 = sm.lookup_token(210, 0).unwrap();

  // assert_eq!(token1.get_src_line(), 3);
  // assert_eq!(token1.get_src_col(), 2);
  // assert_eq!(token2.get_src_line(), 9);
  // assert_eq!(token2.get_src_col(), 2);
}
