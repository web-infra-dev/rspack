#![feature(box_patterns)]

use std::sync::Arc;

use rspack_regex::RspackRegex;
use swc_core::{
  common::{FileName, FilePathMapping, SourceMap},
  ecma::{
    ast::{CallExpr, Expr, Lit},
    parser::parse_file_as_expr,
    visit::swc_ecma_ast,
  },
};
use swc_error_reporters::handler::try_with_handler;

#[cfg(test)]
mod test_regex {

  use rspack_regex::RspackRegex;

  use super::*;

  #[test]
  fn test_basic() {
    // should not panic

    assert!(RspackRegex::with_flags("test\\\\", "").is_ok());
  }
  #[test]
  fn test_js_regex() {
    regex_assert("assert_match(/test.*/,'testaaaaaaa')");
    regex_assert("assert_none_match(/test.*/,'tesaaaaaaa')");
  }
}

enum AssertType {
  False,
  True,
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
    .expect("TODO:");
    match e {
      box Expr::Call(CallExpr { callee, args, .. }) => {
        let t = callee
          .as_expr()
          .and_then(|expr| expr.as_ident())
          .map(|ident| match ident.sym.as_ref() {
            "assert_match" => AssertType::True,
            "assert_none_match" => AssertType::False,
            _ => unimplemented!("unsupported assert function"),
          })
          .expect("TODO:");
        let regex = &args[0];
        let string = &args[1];
        let string_lit = match string.expr.as_lit() {
          Some(Lit::Str(str)) => str.value.to_string(),
          _ => unreachable!(),
        };
        if let Some(Lit::Regex(reg)) = regex.expr.as_lit() {
          let rspack_reg = RspackRegex::try_from(reg.clone()).expect("TODO:");
          match t {
            AssertType::False => assert!(rspack_reg.find(&string_lit).is_none()),
            AssertType::True => assert!(rspack_reg.find(&string_lit).is_some()),
          }
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
