use rspack_core::ConstDependency;
use rspack_plugin_javascript::{JavascriptParserPlugin, visitors::JavascriptParser};
use swc_experimental_ecma_ast::{Ast, Expr, GetSpan, Lit, ModuleItem, Program, Stmt};

pub struct ReactDirectivesParserPlugin;

impl ReactDirectivesParserPlugin {
  fn process_statements<I>(
    ast: &Ast,
    stmts: I,
    directives: &mut Vec<(String, swc_core::common::Span)>,
  ) where
    I: Iterator<Item = Stmt>,
  {
    for stmt in stmts {
      let Stmt::Expr(expr_stmt) = stmt else { break };
      let Expr::Lit(Lit::Str(str_lit)) = expr_stmt.expr(ast) else {
        break;
      };

      let value = ast
        .get_wtf8(str_lit.value(ast))
        .to_string_lossy()
        .to_string();
      if !value.starts_with("use ") {
        break;
      }

      let directive = format!("\"{value}\"");
      directives.push((directive, expr_stmt.span(ast)));
    }
  }
}

impl JavascriptParserPlugin for ReactDirectivesParserPlugin {
  fn program(&self, parser: &mut JavascriptParser, ast: Program) -> Option<bool> {
    let mut directives = Vec::new();

    match ast {
      Program::Module(module) => {
        let stmts = module
          .body(&parser.ast)
          .iter()
          .filter_map(|item| {
            let item = parser.ast.get_node_in_sub_range(item);
            if let ModuleItem::Stmt(stmt) = item {
              Some(stmt)
            } else {
              None
            }
          })
          .collect::<Vec<_>>();
        Self::process_statements(&parser.ast, stmts.into_iter(), &mut directives);
      }
      Program::Script(script) => {
        let stmts: Vec<_> = script
          .body(&parser.ast)
          .iter()
          .map(|i| parser.ast.get_node_in_sub_range(i))
          .collect();
        Self::process_statements(&parser.ast, stmts.into_iter(), &mut directives);
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
