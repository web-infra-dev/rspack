use rspack_core::ConstDependency;
use rspack_plugin_javascript::{JavascriptParserPlugin, visitors::JavascriptParser};
use swc_core::ecma::ast::{Expr, Lit, ModuleItem, Program, Stmt};

pub(crate) struct ReactDirectivesParserPlugin;

impl ReactDirectivesParserPlugin {
  fn process_statements<'a, I>(stmts: I, directives: &mut Vec<(String, swc_core::common::Span)>)
  where
    I: Iterator<Item = &'a Stmt>,
  {
    for stmt in stmts {
      let Stmt::Expr(expr_stmt) = stmt else { break };
      let Expr::Lit(Lit::Str(str_lit)) = &*expr_stmt.expr else {
        break;
      };

      let value = str_lit.value.to_string_lossy().to_string();
      if !value.starts_with("use ") {
        break;
      }

      let directive = format!("\"{value}\"");
      directives.push((directive, expr_stmt.span));
    }
  }
}

impl JavascriptParserPlugin for ReactDirectivesParserPlugin {
  fn program(&self, parser: &mut JavascriptParser, ast: &Program) -> Option<bool> {
    let mut directives = Vec::new();

    match ast {
      Program::Module(module) => {
        let stmts = module.body.iter().filter_map(|item| {
          if let ModuleItem::Stmt(stmt) = item {
            Some(stmt)
          } else {
            None
          }
        });
        Self::process_statements(stmts, &mut directives);
      }
      Program::Script(script) => {
        Self::process_statements(script.body.iter(), &mut directives);
      }
    }

    if directives.is_empty() {
      return None;
    }

    parser.build_info.extras.insert(
      "react_directives".to_string(),
      serde_json::json!(directives.iter().map(|(d, _)| d).collect::<Vec<_>>()),
    );

    for (_, span) in directives {
      parser.add_presentational_dependency(Box::new(ConstDependency::new(span.into(), "".into())));
    }

    None
  }
}
