use std::sync::Arc;

use swc_core::{
  base::try_with_handler,
  common::{errors::Handler, FileName, FilePathMapping, SourceMap},
  ecma::{
    ast::CallExpr,
    parser::{self, parse_file_as_expr},
    visit::swc_ecma_ast,
  },
};

#[cfg(test)]
mod test_super {
  use std::str::FromStr;

  use rspack_regex::RspackRegex;

  use super::*;

  #[test]
  fn test_basic() {
    // should not panic
    let _ = RspackRegex::with_flags("test\\\\", "").unwrap();
  }
}

fn regex_assert(code: &str) {
  let cm = Arc::new(SourceMap::new(FilePathMapping::empty()));
  let fm = cm.new_source_file(FileName::Anon, "test.js".to_string());
  try_with_handler(cm, Default::default(), |handler| {
    let mut e = parse_file_as_expr(
      &fm,
      Default::default(),
      swc_ecma_ast::EsVersion::latest(),
      None,
      &mut vec![],
    )
    .unwrap();
    match e {
      box CallExpr { callee, args, .. } => {}
      _ => {
        unreachable!()
      }
    }
    Ok(())
  });
}
