mod rspack_context;
mod rspack_export;
mod webpack;

use std::borrow::Cow;

use rspack_core::{
  Chunk, ChunkUkey, Compilation, ImportSpec, RuntimeCodeTemplate, RuntimeGlobals,
  RuntimeGlobalsRenderMode, RuntimeVariable, render_imports,
  rspack_sources::{ConcatSource, RawStringSource},
};
use rspack_plugin_javascript::JsPlugin;
use rspack_util::{atom::Atom, fx_hash::FxIndexSet};

use self::{
  rspack_context::RspackContextRuntimeRenderer, rspack_export::RspackExportRuntimeRenderer,
  webpack::WebpackRuntimeRenderer,
};
use crate::chunk_link::{ChunkLinkContext, RawImportSource};

static WEBPACK_RENDERER: WebpackRuntimeRenderer = WebpackRuntimeRenderer;
static RSPACK_CONTEXT_RENDERER: RspackContextRuntimeRenderer = RspackContextRuntimeRenderer;
static RSPACK_EXPORT_RENDERER: RspackExportRuntimeRenderer = RspackExportRuntimeRenderer;

pub(super) fn renderer_for(
  render_mode: RuntimeGlobalsRenderMode,
) -> &'static dyn RuntimeModeRenderer {
  match render_mode {
    RuntimeGlobalsRenderMode::Webpack => &WEBPACK_RENDERER,
    RuntimeGlobalsRenderMode::RspackContext => &RSPACK_CONTEXT_RENDERER,
    RuntimeGlobalsRenderMode::RspackExport => &RSPACK_EXPORT_RENDERER,
    RuntimeGlobalsRenderMode::RspackLexical => {
      unreachable!("chunk code templates never use lexical runtime globals")
    }
  }
}

#[derive(Clone, Copy)]
pub(super) struct RuntimeImportRenderContext<'a> {
  pub compilation: &'a Compilation,
  pub chunk_ukey: &'a ChunkUkey,
  pub runtime_chunk_ukey: &'a ChunkUkey,
  pub chunk_link: &'a ChunkLinkContext,
  pub runtime_requirements: RuntimeGlobals,
  pub runtime_template: &'a RuntimeCodeTemplate,
}

#[derive(Clone, Copy)]
pub(super) struct RuntimeRenderContext<'a> {
  pub chunk_ukey: &'a ChunkUkey,
  pub compilation: &'a Compilation,
  pub runtime_requirements: RuntimeGlobals,
  pub runtime_template: &'a RuntimeCodeTemplate,
  pub should_export_require: bool,
}

pub(super) trait RuntimeModeRenderer: Sync {
  fn render_module_registration_ident(&self, runtime_template: &RuntimeCodeTemplate) -> String;

  fn render_runtime_imports(&self, context: RuntimeImportRenderContext<'_>) -> ConcatSource;

  fn render_direct_runtime_export(
    &self,
    _runtime_template: &RuntimeCodeTemplate,
    _is_pure_runtime_chunk: bool,
    _should_export_require_from_runtime: bool,
  ) -> Option<String> {
    None
  }

  fn renders_inline_runtime_exports(
    &self,
    _compilation: &Compilation,
    _chunk_ukey: &ChunkUkey,
  ) -> bool {
    false
  }

  fn render_runtime(&self, context: RuntimeRenderContext<'_>) -> ConcatSource;
}

fn get_chunk(compilation: &Compilation, chunk_ukey: ChunkUkey) -> &Chunk {
  compilation
    .build_chunk_graph_artifact
    .chunk_by_ukey
    .expect_get(&chunk_ukey)
}

fn normalize_raw_import_source(source: &str) -> Cow<'_, str> {
  let mut value = source.to_string();
  let mut changed = false;

  for _ in 0..2 {
    let decode_target = if value.starts_with("\\\"") && value.ends_with("\\\"") {
      format!("\"{value}\"")
    } else {
      value.clone()
    };
    if let Ok(next) = serde_json::from_str::<String>(&decode_target) {
      changed = true;
      value = next;
      continue;
    }
    break;
  }

  if value.starts_with('\"') && value.ends_with('\"') && value.len() >= 2 {
    changed = true;
    value = value[1..value.len() - 1].to_string();
  }

  if !changed {
    Cow::Borrowed(source)
  } else {
    Cow::Owned(value)
  }
}

fn import_spec_imports_any(import_spec: &ImportSpec, idents: &FxIndexSet<String>) -> bool {
  let is_runtime_import = |local: &Atom| {
    idents
      .iter()
      .any(|runtime_import_ident| local.as_str() == runtime_import_ident)
  };
  import_spec.atoms.values().any(&is_runtime_import)
    || import_spec
      .default_import
      .as_ref()
      .is_some_and(is_runtime_import)
    || import_spec
      .ns_import
      .as_ref()
      .is_some_and(is_runtime_import)
}

fn already_imports_runtime(
  chunk_link: &ChunkLinkContext,
  runtime_import_idents: &FxIndexSet<String>,
) -> bool {
  chunk_link
    .raw_import_stmts
    .iter()
    .any(|(raw_import_source, import_spec)| {
      matches!(raw_import_source, RawImportSource::Chunk(_))
        && import_spec_imports_any(import_spec, runtime_import_idents)
    })
}

fn render_runtime_chunk_import(
  compilation: &Compilation,
  runtime_chunk_ukey: &ChunkUkey,
  runtime_import_idents: &FxIndexSet<String>,
) -> ConcatSource {
  let mut source = ConcatSource::default();
  if runtime_import_idents.is_empty() {
    return source;
  }

  let runtime_chunk = get_chunk(compilation, *runtime_chunk_ukey);
  source.add(RawStringSource::from(format!(
    "import {{ {} }} from \"__RSPACK_ESM_CHUNK_{}\";\n",
    runtime_import_idents
      .iter()
      .map(String::as_str)
      .collect::<Vec<_>>()
      .join(", "),
    runtime_chunk.expect_id().as_str()
  )));
  source
}

fn render_raw_import_stmts(
  compilation: &Compilation,
  chunk_link: &ChunkLinkContext,
  runtime_chunk_ukey: Option<&ChunkUkey>,
  runtime_import_match_idents: &FxIndexSet<String>,
  legacy_context_idents: Option<&FxIndexSet<String>>,
) -> ConcatSource {
  let mut source = ConcatSource::default();
  for (raw_import_source, import_spec) in &chunk_link.raw_import_stmts {
    let should_skip = match raw_import_source {
      RawImportSource::Chunk(import_chunk) => {
        runtime_chunk_ukey == Some(import_chunk)
          && legacy_context_idents
            .is_some_and(|idents| import_spec_imports_any(import_spec, idents))
      }
      RawImportSource::Source((request, _)) if request.contains("__RSPACK_ESM_CHUNK_") => {
        import_spec_imports_any(import_spec, runtime_import_match_idents)
          || legacy_context_idents
            .is_some_and(|idents| import_spec_imports_any(import_spec, idents))
      }
      _ => false,
    };
    if should_skip {
      continue;
    }

    let (request, attr) = match raw_import_source {
      RawImportSource::Chunk(import_chunk) => {
        let chunk = get_chunk(compilation, *import_chunk);
        (
          Cow::Owned(format!("__RSPACK_ESM_CHUNK_{}", chunk.expect_id().as_str())),
          None,
        )
      }
      RawImportSource::Source((request, attr)) => (
        normalize_raw_import_source(request.as_str()),
        attr.as_deref(),
      ),
    };

    source.add(RawStringSource::from(render_imports(
      &request,
      attr,
      import_spec,
    )));
  }
  source
}

fn render_runtime_prelude(context: RuntimeRenderContext<'_>) -> ConcatSource {
  let require_function = context
    .runtime_requirements
    .contains(RuntimeGlobals::REQUIRE);
  let module_cache = context
    .runtime_requirements
    .contains(RuntimeGlobals::MODULE_CACHE);
  let intercept_module_execution = context
    .runtime_requirements
    .contains(RuntimeGlobals::INTERCEPT_MODULE_EXECUTION);
  let module_used = context
    .runtime_requirements
    .contains(RuntimeGlobals::MODULE);
  let require_scope_used = context
    .runtime_requirements
    .contains(RuntimeGlobals::REQUIRE_SCOPE);
  let use_require = require_function || intercept_module_execution || module_used;
  let mut source = ConcatSource::default();

  if use_require || module_cache {
    source.add(RawStringSource::from(format!(
      r#"// The module cache
var {} = {{}};
"#,
      context
        .runtime_template
        .render_runtime_variable(&RuntimeVariable::ModuleCache)
    )));
  }

  if use_require {
    let require = context
      .runtime_template
      .render_runtime_variable(&RuntimeVariable::Require);
    source.add(RawStringSource::from(format!(
      r#"// The require function
function {require}(moduleId) {{
"#,
    )));
    source.add(RawStringSource::from(
      JsPlugin::render_require(
        context.chunk_ukey,
        context.compilation,
        context.runtime_template,
      )
      .join("\n"),
    ));
    source.add(RawStringSource::from_static(
      r#"
}
"#,
    ));
  } else if require_scope_used {
    let require = context
      .runtime_template
      .render_runtime_variable(&RuntimeVariable::Require);
    source.add(RawStringSource::from(format!(
      r#"// The require scope
var {require} = {{}};
"#,
    )));
  }

  source
}

fn render_module_factories_definition(runtime_template: &RuntimeCodeTemplate) -> RawStringSource {
  let module_factories =
    runtime_template.render_runtime_global_definition(&RuntimeGlobals::MODULE_FACTORIES);
  RawStringSource::from(format!(
    r#"// expose the modules object ({modules})
{module_factories} = {modules};
"#,
    modules = runtime_template.render_runtime_variable(&RuntimeVariable::Modules),
  ))
}

fn render_module_cache_definition(runtime_template: &RuntimeCodeTemplate) -> RawStringSource {
  let module_cache_name = runtime_template.render_runtime_globals(&RuntimeGlobals::MODULE_CACHE);
  let runtime_module_cache =
    runtime_template.render_runtime_variable(&RuntimeVariable::ModuleCache);
  if module_cache_name == runtime_module_cache {
    return RawStringSource::from_static("");
  }
  let module_cache =
    runtime_template.render_runtime_global_definition(&RuntimeGlobals::MODULE_CACHE);
  RawStringSource::from(format!(
    r#"// expose the module cache
{module_cache} = {runtime_module_cache};
"#,
  ))
}

fn render_intercept_module_execution_definition(
  runtime_template: &RuntimeCodeTemplate,
) -> RawStringSource {
  let intercept_module_execution =
    runtime_template.render_runtime_global_definition(&RuntimeGlobals::INTERCEPT_MODULE_EXECUTION);
  RawStringSource::from(format!(
    r#"// expose the module execution interceptor
{intercept_module_execution} = [];
"#,
  ))
}

fn render_runtime_global_definitions(context: RuntimeRenderContext<'_>) -> ConcatSource {
  let mut source = ConcatSource::default();
  if context
    .runtime_requirements
    .contains(RuntimeGlobals::MODULE_FACTORIES)
  {
    source.add(render_module_factories_definition(context.runtime_template));
  }
  if context
    .runtime_requirements
    .contains(RuntimeGlobals::MODULE_CACHE)
  {
    source.add(render_module_cache_definition(context.runtime_template));
  }
  if context
    .runtime_requirements
    .contains(RuntimeGlobals::INTERCEPT_MODULE_EXECUTION)
  {
    source.add(render_intercept_module_execution_definition(
      context.runtime_template,
    ));
  }
  source
}
