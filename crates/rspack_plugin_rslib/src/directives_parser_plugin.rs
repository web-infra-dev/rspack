use rspack_core::ConstDependency;
use rspack_plugin_javascript::{JavascriptParserPlugin, visitors::JavascriptParser};
use swc_core::ecma::ast::{Expr, Lit, ModuleItem, Program, Stmt};

pub struct DirectivesParserPlugin;

impl DirectivesParserPlugin {
  fn process_statements<'a, I>(stmts: I, directives: &mut Vec<(String, swc_core::common::Span)>)
  where
    I: Iterator<Item = &'a Stmt>,
  {
    for stmt in stmts {
      if let Stmt::Expr(expr_stmt) = stmt {
        if let Expr::Lit(Lit::Str(str_lit)) = &*expr_stmt.expr {
          let value = str_lit.value.to_string_lossy().to_string();
          // Check if it starts with "use "
          if value.starts_with("use ") {
            // Store the directive with its original quotes
            let quote = if str_lit.raw.as_ref().is_some_and(|r| r.starts_with('\'')) {
              '\''
            } else {
              '"'
            };
            let directive = format!("{}{}{}", quote, value, quote);
            directives.push((directive, expr_stmt.span));
          } else {
            // Stop at the first non-directive statement
            break;
          }
        } else {
          // Stop at the first non-string-literal statement
          break;
        }
      } else {
        // Stop at the first non-expression statement
        break;
      }
    }
  }
}

impl JavascriptParserPlugin for DirectivesParserPlugin {
  fn program(&self, parser: &mut JavascriptParser, ast: &Program) -> Option<bool> {
    let mut directives = Vec::new();

    // Get items based on whether it's a module or script
    match ast {
      Program::Module(module) => {
        // Process module items, converting them to statements
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
        // Process script statements
        Self::process_statements(script.body.iter(), &mut directives);
      }
    }

    if directives.is_empty() {
      return None;
    }

    // Store directives in build_info
    parser.build_info.extras.insert(
      "directives".to_string(),
      serde_json::json!(directives.iter().map(|(d, _)| d).collect::<Vec<_>>()),
    );

    // Remove directives from source code
    for (_, span) in directives {
      parser.add_presentational_dependency(Box::new(ConstDependency::new(
        (span.lo.0 - 1, span.hi.0 - 1).into(),
        "".into(),
        None,
      )));
    }

    None
  }
}
