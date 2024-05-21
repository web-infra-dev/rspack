use std::collections::HashMap;

use once_cell::sync::OnceCell;
use rspack_core::{ConstDependency, Plugin, SpanExt};
use swc_core::common::Spanned;

use crate::parser_plugin::JavascriptParserPlugin;

type DefineValue = HashMap<String, String>;

#[derive(Debug, Default)]
pub struct DefinePlugin {
  cached_names: OnceCell<Vec<String>>,
}

impl Plugin for DefinePlugin {
  fn name(&self) -> &'static str {
    "rspack.DefinePlugin"
  }
}

fn dep(
  parser: &mut crate::visitors::JavascriptParser,
  for_name: &str,
  definitions: &DefineValue,
  start: u32,
  end: u32,
) -> Option<ConstDependency> {
  if let Some(value) = definitions.get(for_name) {
    if parser.in_short_hand {
      return Some(ConstDependency::new(
        start,
        end,
        format!("{for_name}: {value}").into(),
        None,
      ));
    } else {
      return Some(ConstDependency::new(
        start,
        end,
        value.to_string().into(),
        None,
      ));
    }
  }
  None
}

impl JavascriptParserPlugin for DefinePlugin {
  fn can_rename(&self, parser: &mut crate::visitors::JavascriptParser, str: &str) -> Option<bool> {
    let names = self.cached_names.get_or_init(|| {
      let names = parser.compiler_options.builtins.define.keys();
      names
        .flat_map(|name| {
          let splitted: Vec<&str> = name.split('.').collect();
          let mut val = if !splitted.is_empty() {
            (0..splitted.len() - 1)
              .map(|i| splitted[0..i + 1].join("."))
              .collect::<Vec<_>>()
          } else {
            vec![]
          };
          // !isTypeof
          val.push(name.to_string());
          val
        })
        .collect::<Vec<_>>()
    });

    if names.iter().any(|l| *l == str) {
      return Some(true);
    }

    None
  }

  fn evaluate_identifier(
    &self,
    parser: &mut crate::visitors::JavascriptParser,
    ident: &str,
    start: u32,
    end: u32,
  ) -> Option<crate::utils::eval::BasicEvaluatedExpression> {
    if let Some(val) = parser.compiler_options.builtins.define.get(ident) {
      return parser
        .evaluate(val.to_string(), "DefinePlugin".to_string())
        .map(|mut evaluated| {
          evaluated.set_range(start, end);
          evaluated
        });
    }
    None
  }

  fn call(
    &self,
    parser: &mut crate::visitors::JavascriptParser,
    expr: &swc_core::ecma::ast::CallExpr,
    for_name: &str,
  ) -> Option<bool> {
    dep(
      parser,
      for_name,
      &parser.compiler_options.builtins.define,
      expr.callee.span().real_lo(),
      expr.callee.span().real_hi(),
    )
    .map(|dep| {
      parser.presentational_dependencies.push(Box::new(dep));
      // FIXME: webpack use `walk_expression` here
      parser.walk_expr_or_spread(&expr.args);
      true
    })
  }

  fn member(
    &self,
    parser: &mut crate::visitors::JavascriptParser,
    expr: &swc_core::ecma::ast::MemberExpr,
    for_name: &str,
  ) -> Option<bool> {
    dep(
      parser,
      for_name,
      &parser.compiler_options.builtins.define,
      expr.span().real_lo(),
      expr.span().real_hi(),
    )
    .map(|dep| {
      parser.presentational_dependencies.push(Box::new(dep));
      true
    })
  }

  fn identifier(
    &self,
    parser: &mut crate::visitors::JavascriptParser,
    ident: &swc_core::ecma::ast::Ident,
    for_name: &str,
  ) -> Option<bool> {
    dep(
      parser,
      for_name,
      &parser.compiler_options.builtins.define,
      ident.span.real_lo(),
      ident.span.real_hi(),
    )
    .map(|dep| {
      parser.presentational_dependencies.push(Box::new(dep));
      true
    })
  }
}
