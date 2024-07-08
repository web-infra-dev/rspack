use std::collections::HashMap;

use once_cell::sync::OnceCell;
use rspack_core::{
  ApplyContext, Compilation, CompilationParams, CompilerCompilation, CompilerOptions,
  ConstDependency, ModuleType, NormalModuleFactoryParser, ParserAndGenerator, ParserOptions,
  Plugin, PluginContext, SpanExt,
};
use rspack_error::{
  miette::{self, Diagnostic},
  thiserror::{self, Error},
  DiagnosticExt, Result,
};
use rspack_hook::{plugin, plugin_hook};
use swc_core::common::Spanned;

use crate::{
  parser_and_generator::JavaScriptParserAndGenerator, parser_plugin::JavascriptParserPlugin,
  visitors::JavascriptParser, BoxJavascriptParserPlugin,
};

type DefineValue = HashMap<String, String>;

#[plugin]
#[derive(Debug, Default, Clone)]
pub struct DefinePlugin {
  definitions: DefineValue,
  cached_names: OnceCell<Vec<String>>,
}

impl DefinePlugin {
  pub fn new(definitions: DefineValue) -> Self {
    Self::new_inner(definitions, OnceCell::new())
  }

  fn cached_names(&self) -> &Vec<String> {
    self.cached_names.get_or_init(|| {
      let names = self.definitions.keys();
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
        .collect()
    })
  }
}

#[derive(Debug, Error, Diagnostic)]
#[error("DefinePlugin:\nConflicting values for '{0}' ('{1}' !== '{2}')")]
#[diagnostic(severity(Warning))]
struct ConflictingValuesError(String, String, String);

#[plugin_hook(CompilerCompilation for DefinePlugin)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  self.definitions.iter().for_each(|(key, value)| {
    let name = format!("{VALUE_DEP_PREFIX}{key}");
    if let Some(prev) = compilation.value_cache_versions.get(&name)
      && prev != value
    {
      compilation.push_diagnostic(
        ConflictingValuesError(key.to_string(), prev.clone(), value.clone())
          .boxed()
          .into(),
      );
    } else {
      compilation.value_cache_versions.insert(name, value.clone());
    }
  });
  Ok(())
}

#[plugin_hook(NormalModuleFactoryParser for DefinePlugin)]
fn nmf_parser(
  &self,
  module_type: &ModuleType,
  parser: &mut dyn ParserAndGenerator,
  _parser_options: Option<&ParserOptions>,
) -> Result<()> {
  if module_type.is_js_like()
    && let Some(parser) = parser.downcast_mut::<JavaScriptParserAndGenerator>()
  {
    parser.add_parser_plugin(Box::new(self.clone()) as BoxJavascriptParserPlugin);
  }
  Ok(())
}

impl Plugin for DefinePlugin {
  fn name(&self) -> &'static str {
    "rspack.DefinePlugin"
  }

  fn apply(
    &self,
    ctx: PluginContext<&mut ApplyContext>,
    _options: &mut CompilerOptions,
  ) -> Result<()> {
    ctx
      .context
      .compiler_hooks
      .compilation
      .tap(compilation::new(self));
    ctx
      .context
      .normal_module_factory_hooks
      .parser
      .tap(nmf_parser::new(self));
    Ok(())
  }
}

fn dep(
  parser: &mut JavascriptParser,
  for_name: &str,
  definitions: &DefineValue,
  start: u32,
  end: u32,
  asi_safe: bool,
) -> Option<ConstDependency> {
  if let Some(value) = definitions.get(for_name) {
    let code = if parser.in_short_hand {
      format!("{for_name}: {value}")
    } else if asi_safe {
      format!("({value})")
    } else {
      format!(";({value})")
    };

    return Some(ConstDependency::new(start, end, code.into(), None));
  }
  None
}

const VALUE_DEP_PREFIX: &str = "webpack/DefinePlugin ";

impl JavascriptParserPlugin for DefinePlugin {
  fn can_rename(&self, _: &mut JavascriptParser, str: &str) -> Option<bool> {
    let names = self.cached_names();
    if names.iter().any(|l: &String| l.eq(str)) {
      return Some(true);
    }
    None
  }

  fn evaluate_identifier(
    &self,
    parser: &mut JavascriptParser,
    ident: &str,
    start: u32,
    end: u32,
  ) -> Option<crate::utils::eval::BasicEvaluatedExpression> {
    if let Some(val) = self.definitions.get(ident) {
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
    parser: &mut JavascriptParser,
    expr: &swc_core::ecma::ast::CallExpr,
    for_name: &str,
  ) -> Option<bool> {
    dep(
      parser,
      for_name,
      &self.definitions,
      expr.callee.span().real_lo(),
      expr.callee.span().real_hi(),
      !parser.is_asi_position(expr.span_lo()),
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
    parser: &mut JavascriptParser,
    expr: &swc_core::ecma::ast::MemberExpr,
    for_name: &str,
  ) -> Option<bool> {
    dep(
      parser,
      for_name,
      &self.definitions,
      expr.span().real_lo(),
      expr.span().real_hi(),
      !parser.is_asi_position(expr.span_lo()),
    )
    .map(|dep| {
      parser.presentational_dependencies.push(Box::new(dep));
      true
    })
  }

  fn identifier(
    &self,
    parser: &mut JavascriptParser,
    ident: &swc_core::ecma::ast::Ident,
    for_name: &str,
  ) -> Option<bool> {
    dep(
      parser,
      for_name,
      &self.definitions,
      ident.span.real_lo(),
      ident.span.real_hi(),
      !parser.is_asi_position(ident.span_lo()),
    )
    .map(|dep| {
      parser.presentational_dependencies.push(Box::new(dep));
      true
    })
  }
}
