mod parser;
mod utils;
mod walk_data;

use std::{collections::HashMap, sync::Arc};

use parser::DefineParserPlugin;
use rspack_core::{
  Compilation, CompilationParams, CompilerCompilation, ModuleType, NormalModuleFactoryParser,
  ParserAndGenerator, ParserOptions, Plugin,
};
use rspack_error::{
  DiagnosticExt, Result,
  miette::{self, Diagnostic as MietteDiagnostic},
  thiserror::{self, Error},
};
use rspack_hook::{plugin, plugin_hook};
use serde_json::Value;

use self::walk_data::WalkData;
use crate::parser_and_generator::JavaScriptParserAndGenerator;

const VALUE_DEP_PREFIX: &str = "rspack/DefinePlugin ";

#[derive(Debug, Error, MietteDiagnostic)]
#[error("DefinePlugin:\nConflicting values for '{0}' ({1} !== {2})")]
#[diagnostic(severity(Warning))]
struct ConflictingValuesError(String, String, String);

pub type DefineValue = HashMap<String, Value>;

#[plugin]
#[derive(Debug)]
pub struct DefinePlugin {
  walk_data: Arc<WalkData>,
}

impl DefinePlugin {
  pub fn new(definitions: DefineValue) -> Self {
    Self::new_inner(Arc::new(WalkData::new(&definitions)))
  }
}

#[plugin_hook(CompilerCompilation for DefinePlugin, tracing=false)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  compilation.extend_diagnostics(self.walk_data.diagnostics.clone());
  for (key, value) in self.walk_data.tiling_definitions.iter() {
    let cache_key = format!("{VALUE_DEP_PREFIX}{key}");
    if let Some(prev) = compilation.value_cache_versions.get(&cache_key)
      && prev != value
    {
      compilation.push_diagnostic(
        ConflictingValuesError(key.clone(), prev.clone(), value.clone())
          .boxed()
          .into(),
      );
    } else {
      compilation
        .value_cache_versions
        .insert(cache_key, value.clone());
    }
  }

  Ok(())
}

#[plugin_hook(NormalModuleFactoryParser for DefinePlugin, tracing=false)]
async fn nmf_parser(
  &self,
  module_type: &ModuleType,
  parser: &mut dyn ParserAndGenerator,
  _parser_options: Option<&ParserOptions>,
) -> Result<()> {
  if module_type.is_js_like()
    && let Some(parser) = parser.downcast_mut::<JavaScriptParserAndGenerator>()
  {
    parser.add_parser_plugin(Box::new(DefineParserPlugin::new(self.walk_data.clone())));
  }
  Ok(())
}

impl Plugin for DefinePlugin {
  fn name(&self) -> &'static str {
    "rspack.DefinePlugin"
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
