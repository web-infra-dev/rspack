mod parser;

use std::sync::Arc;

use rspack_core::{
  Compilation, CompilationParams, CompilerCompilation, ModuleType, NormalModuleFactoryParser,
  ParserAndGenerator, ParserOptions, Plugin,
};
use rspack_error::{Diagnostic, Error, Result};
use rspack_hook::{plugin, plugin_hook};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use self::parser::ProvideParserPlugin;
use crate::parser_and_generator::JavaScriptParserAndGenerator;

const VALUE_DEP_PREFIX: &str = "rspack/ProvidePlugin ";
type ProvideValue = HashMap<String, Vec<String>>;

#[derive(Debug)]
struct ConflictingValuesError(String, String, String);

impl ConflictingValuesError {
  fn into_diagnostic(self) -> Diagnostic {
    Error::warning(format!(
      "ProvidePlugin:\nConflicting values for '{}' ({} !== {})",
      self.0, self.1, self.2
    ))
    .into()
  }
}

#[plugin]
#[derive(Default, Debug, Clone)]
pub struct ProvidePlugin {
  provide: Arc<ProvideValue>,
  names: Arc<HashSet<String>>,
}

impl ProvidePlugin {
  pub fn new(provide: ProvideValue) -> Self {
    let names = provide
      .keys()
      .flat_map(|name| {
        let splitted: Vec<&str> = name.split('.').collect();
        // splitted.len() is always greater than 0
        (0..splitted.len() - 1)
          .map(|i| splitted[0..i + 1].join("."))
          .collect::<Vec<_>>()
      })
      .collect::<HashSet<_>>();
    Self::new_inner(provide.into(), names.into())
  }
}

#[plugin_hook(CompilerCompilation for ProvidePlugin, tracing=false)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  for (key, value) in self.provide.iter() {
    let cache_key = format!("{VALUE_DEP_PREFIX}{key}");
    let value = value.join(".");
    if let Some(prev) = compilation.value_cache_versions.get(&cache_key)
      && prev != &value
    {
      compilation.push_diagnostic(
        ConflictingValuesError(key.clone(), prev.clone(), value).into_diagnostic(),
      );
    } else {
      compilation.value_cache_versions.insert(cache_key, value);
    }
  }

  Ok(())
}

#[plugin_hook(NormalModuleFactoryParser for ProvidePlugin)]
async fn nmf_parser(
  &self,
  module_type: &ModuleType,
  parser: &mut Box<dyn ParserAndGenerator>,
  _parser_options: Option<&ParserOptions>,
) -> Result<()> {
  if module_type.is_js_like()
    && let Some(parser) = parser.downcast_mut::<JavaScriptParserAndGenerator>()
  {
    parser.add_parser_plugin(Box::new(ProvideParserPlugin::new(
      self.provide.clone(),
      self.names.clone(),
    )));
  }
  Ok(())
}

impl Plugin for ProvidePlugin {
  fn name(&self) -> &'static str {
    "rspack.ProvidePlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compiler_hooks.compilation.tap(compilation::new(self));
    ctx
      .normal_module_factory_hooks
      .parser
      .tap(nmf_parser::new(self));
    Ok(())
  }
}
