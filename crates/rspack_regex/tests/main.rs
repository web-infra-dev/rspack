#![feature(box_patterns)]

use std::sync::Arc;

use rspack_regex::RspackRegex;
use swc_core::{
  base::try_with_handler,
  common::{FileName, FilePathMapping, SourceMap},
  ecma::{
    ast::{CallExpr, Expr, Lit},
    parser::parse_file_as_expr,
    visit::swc_ecma_ast,
  },
};

#[cfg(test)]
mod test_super {

  use rspack_regex::RspackRegex;

  use super::*;

  #[test]
  fn test_basic() {
    // should not panic
    let _ = RspackRegex::with_flags("test\\\\", "").unwrap();
  }
  #[test]
  fn test_js_regex() {
    // should not panic
    regex_assert("assert_regex(/test.*/,'testaaaaaaa')");
  }
}

fn regex_assert(code: &str) {
  let cm = Arc::new(SourceMap::new(FilePathMapping::empty()));
  let fm = cm.new_source_file(FileName::Anon, code.to_string());
  _ = try_with_handler(cm, Default::default(), |_| {
    let e = parse_file_as_expr(
      &fm,
      Default::default(),
      swc_ecma_ast::EsVersion::latest(),
      None,
      &mut vec![],
    )
    .unwrap();
    match e {
      box Expr::Call(CallExpr { callee, args, .. }) => {
        let t = callee
          .as_expr()
          .and_then(|expr| expr.as_ident())
          .map(|ident| &ident.sym == "assert_regex");
        assert_eq!(t, Some(true));
        let regex = &args[0];
        let string = &args[1];
        let string_lit = match string.expr.as_lit() {
          Some(Lit::Str(str)) => str.value.to_string(),
          _ => unreachable!(),
        };
        if let Some(Lit::Regex(reg)) = regex.expr.as_lit() {
          let rspack_reg = RspackRegex::try_from(reg.clone()).unwrap();

          assert!(rspack_reg.find(&string_lit).is_some());
        } else {
          unreachable!()
        }
      }

      _ => {
        unreachable!()
      }
    }
    Ok(())
  });
}
