mod parser;

use std::{borrow::Cow, collections::HashMap};

use itertools::Itertools;
use parser::{walk_definitions, DefineParserPlugin};
use rspack_core::{
  ApplyContext, Compilation, CompilationParams, CompilerCompilation, CompilerOptions, ModuleType,
  NormalModuleFactoryParser, ParserAndGenerator, ParserOptions, Plugin, PluginContext,
};
use rspack_error::{
  miette::{self, Diagnostic},
  thiserror::{self, Error},
  DiagnosticExt, Result,
};
use rspack_hook::{plugin, plugin_hook};
use serde_json::Value;

use crate::parser_and_generator::JavaScriptParserAndGenerator;

type DefineValue = HashMap<String, Value>;

const VALUE_DEP_PREFIX: &str = "webpack/DefinePlugin ";

#[plugin]
#[derive(Debug)]
pub struct DefinePlugin {
  definitions: DefineValue,
}

impl DefinePlugin {
  pub fn new(definitions: DefineValue) -> Self {
    Self::new_inner(definitions)
  }
}

#[derive(Debug, Error, Diagnostic)]
#[error("DefinePlugin:\nConflicting values for '{0}' ({1} !== {2})")]
#[diagnostic(severity(Warning))]
struct ConflictingValuesError(String, String, String);

#[plugin_hook(CompilerCompilation for DefinePlugin)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  fn walk_definitions<'d, 's>(
    definitions: impl Iterator<Item = (&'d String, &'d Value)>,
    compilation: &mut Compilation,
    prefix: Cow<'s, str>,
  ) {
    definitions.for_each(|(key, value)| {
      let name = format!("{VALUE_DEP_PREFIX}{prefix}{key}");
      let value_str = value.to_string();
      if let Some(prev) = compilation.value_cache_versions.get(&name)
        && !prev.eq(&value_str)
      {
        compilation.push_diagnostic(
          ConflictingValuesError(format!("{prefix}{key}"), prev.clone(), value_str)
            .boxed()
            .into(),
        );
      } else {
        compilation.value_cache_versions.insert(name, value_str);
      }
      if let Some(value) = value.as_object() {
        walk_definitions(
          value.iter(),
          compilation,
          Cow::Owned(format!("{prefix}{key}.")),
        )
      } else if let Some(value) = value.as_array() {
        let indexes = (0..value.len())
          .map(|index| format!("{}", index))
          .collect_vec();
        let iter = indexes.iter().zip(value.iter());
        walk_definitions(iter, compilation, Cow::Owned(format!("{prefix}{key}.")))
      }
    });
  }
  walk_definitions(self.definitions.iter(), compilation, "".into());
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
    let walk_data = walk_definitions(&self.definitions);
    parser.add_parser_plugin(Box::new(DefineParserPlugin { walk_data }));
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
