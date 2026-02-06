use swc_core::ecma::{
  ast::{CallExpr, Callee, ExportAll, Expr, ImportDecl, Lit},
  visit::Visit,
};

/// A swc visitor to collect import/require
#[derive(Debug, Default)]
pub(super) struct DependencyVisitor {
  pub requests: Vec<String>,
}

impl Visit for DependencyVisitor {
  /// handle `import .. from "..."`
  fn visit_import_decl(&mut self, node: &ImportDecl) {
    self
      .requests
      .push(node.src.value.to_string_lossy().to_string());
  }

  /// handle `import("...")` and `require("...")`
  fn visit_call_expr(&mut self, node: &CallExpr) {
    let is_match_ident = match &node.callee {
      Callee::Import(_) => true,
      Callee::Expr(expr) if matches!(expr.as_ref(), Expr::Ident(ident) if ident.sym == "require") => {
        true
      }
      _ => false,
    };
    if is_match_ident
      && let Some(args) = node.args.first()
      && let Expr::Lit(Lit::Str(s)) = args.expr.as_ref()
    {
      self.requests.push(s.value.to_string_lossy().to_string());
    }
  }

  /// handle `export * from "..."`
  fn visit_export_all(&mut self, node: &ExportAll) {
    self
      .requests
      .push(node.src.value.to_string_lossy().to_string());
  }

  /// handle `export .. from "..."`
  fn visit_named_export(&mut self, node: &swc_core::ecma::ast::NamedExport) {
    if let Some(src) = &node.src {
      self.requests.push(src.value.to_string_lossy().to_string());
    }
  }
}

#[cfg(test)]
mod test {
  use rspack_javascript_compiler::JavaScriptCompiler;
  use swc_core::{
    base::config::IsModule,
    common::FileName,
    ecma::{ast::EsVersion, parser::Syntax},
  };

  use super::DependencyVisitor;

  #[test]
  fn visitor_test() {
    let javascript_compiler = JavaScriptCompiler::new();
    let source = r#"
import a from "./a";
import { a1 } from "./a";
import { b } from "./b";
import * as b1 from "./b";
import "./c";
import("./d");
import("./d" + 1);
require("./e");
require("./e" + 1);
export * from "./f";
export { g } from "./g";
export { h as default } from "./h";
"#;
    let ast = javascript_compiler
      .parse(
        FileName::Custom(String::new()),
        source,
        EsVersion::EsNext,
        Syntax::Es(Default::default()),
        IsModule::Unknown,
        None,
      )
      .expect("should parse success");
    let mut visitor = DependencyVisitor::default();
    ast.visit(|program, _| {
      program.visit_with(&mut visitor);
    });
    visitor.requests.sort();
    assert_eq!(
      visitor.requests,
      vec![
        "./a", "./a", "./b", "./b", "./c", "./d", "./e", "./f", "./g", "./h"
      ]
    );
  }
}
