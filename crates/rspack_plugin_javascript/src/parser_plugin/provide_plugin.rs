use itertools::Itertools;
use once_cell::sync::OnceCell;
use rspack_core::{
  ApplyContext, CompilerOptions, ModuleType, NormalModuleFactoryParser, ParserAndGenerator,
  ParserOptions, Plugin, PluginContext, SpanExt,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use swc_core::{atoms::Atom, common::Spanned};

use super::JavascriptParserPlugin;
use crate::{
  dependency::ProvideDependency, parser_and_generator::JavaScriptParserAndGenerator,
  visitors::JavascriptParser, BoxJavascriptParserPlugin,
};

const SOURCE_DOT: &str = r#"."#;
const MODULE_DOT: &str = r#"_dot_"#;

fn dep(value: &ProvideValue, name: &str, start: u32, end: u32) -> Option<ProvideDependency> {
  if let Some(requests) = value.get(name) {
    let name_identifier = if name.contains(SOURCE_DOT) {
      format!("__webpack_provide_{}", name.replace(SOURCE_DOT, MODULE_DOT))
    } else {
      name.to_string()
    };
    return Some(ProvideDependency::new(
      start,
      end,
      Atom::from(requests[0].as_str()),
      name_identifier,
      requests[1..]
        .iter()
        .map(|s| Atom::from(s.as_str()))
        .collect_vec(),
    ));
  }
  None
}

type ProvideValue = std::collections::HashMap<String, Vec<String>>;

#[plugin]
#[derive(Default, Debug, Clone)]
pub struct ProvidePlugin {
  provide: ProvideValue,
  cached_names: OnceCell<Vec<String>>,
}

impl ProvidePlugin {
  pub fn new(provide: ProvideValue) -> Self {
    Self::new_inner(provide, OnceCell::new())
  }

  fn cached_names(&self) -> &Vec<String> {
    self.cached_names.get_or_init(|| {
      self
        .provide
        .keys()
        .flat_map(|name| {
          let splitted: Vec<&str> = name.split('.').collect();
          if !splitted.is_empty() {
            (0..splitted.len() - 1)
              .map(|i| splitted[0..i + 1].join("."))
              .collect::<Vec<_>>()
          } else {
            vec![]
          }
        })
        .collect::<Vec<_>>()
    })
  }
}

impl JavascriptParserPlugin for ProvidePlugin {
  fn can_rename(&self, _parser: &mut JavascriptParser, str: &str) -> Option<bool> {
    let names = self.cached_names();
    names.iter().any(|l| l.eq(str)).then_some(true)
  }

  fn call(
    &self,
    parser: &mut JavascriptParser,
    expr: &swc_core::ecma::ast::CallExpr,
    for_name: &str,
  ) -> Option<bool> {
    dep(
      &self.provide,
      for_name,
      expr.callee.span().real_lo(),
      expr.callee.span().real_hi(),
    )
    .map(|dep| {
      parser.dependencies.push(Box::new(dep));
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
      &self.provide,
      for_name,
      expr.span().real_lo(),
      expr.span().real_hi(),
    )
    .map(|dep| {
      parser.dependencies.push(Box::new(dep));
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
      &self.provide,
      for_name,
      ident.span.real_lo(),
      ident.span.real_hi(),
    )
    .map(|dep| {
      parser.dependencies.push(Box::new(dep));
      true
    })
  }
}

#[plugin_hook(NormalModuleFactoryParser for ProvidePlugin)]
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

impl Plugin for ProvidePlugin {
  fn name(&self) -> &'static str {
    "rspack.ProvidePlugin"
  }

  fn apply(
    &self,
    ctx: PluginContext<&mut ApplyContext>,
    _options: &mut CompilerOptions,
  ) -> Result<()> {
    ctx
      .context
      .normal_module_factory_hooks
      .parser
      .tap(nmf_parser::new(self));
    Ok(())
  }
}
