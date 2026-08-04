mod rspack_context;
mod rspack_export;
mod webpack;

use std::borrow::Cow;

use rspack_core::{
  Chunk, ChunkUkey, Compilation, ImportSpec, RuntimeCodeTemplate, RuntimeGlobals,
  RuntimeGlobalsRenderMode, RuntimeVariable, render_imports,
  rspack_sources::{ConcatSource, RawStringSource},
};
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
  fn render_runtime_imports(&self, context: RuntimeImportRenderContext<'_>) -> ConcatSource;

  fn render_direct_runtime_export(
    &self,
    _runtime_template: &RuntimeCodeTemplate,
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

fn render_single_runtime_import(
  context: RuntimeImportRenderContext<'_>,
  runtime_import_ident: String,
  extra_match_ident: Option<String>,
) -> ConcatSource {
  let runtime_import_idents = FxIndexSet::from_iter([runtime_import_ident]);
  let mut runtime_import_match_idents = runtime_import_idents.clone();
  runtime_import_match_idents.extend(extra_match_ident);

  let mut source = ConcatSource::default();
  if !context.runtime_requirements.is_empty()
    && context.runtime_chunk_ukey != context.chunk_ukey
    && context
      .runtime_requirements
      .intersects(RuntimeGlobals::REQUIRE | RuntimeGlobals::REQUIRE_SCOPE)
    && !already_imports_runtime(context.chunk_link, &runtime_import_match_idents)
  {
    source.add(render_runtime_chunk_import(
      context.compilation,
      context.runtime_chunk_ukey,
      &runtime_import_idents,
    ));
  }
  source.add(render_raw_import_stmts(
    context.compilation,
    context.chunk_link,
    None,
    &runtime_import_match_idents,
    None,
  ));
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
  let module_cache = context
    .runtime_requirements
    .contains(RuntimeGlobals::MODULE_CACHE);
  let require_scope_used =
    if context.runtime_template.render_mode() == RuntimeGlobalsRenderMode::RspackExport {
      context
        .runtime_requirements
        .intersects(RuntimeGlobals::REQUIRE | RuntimeGlobals::INTERCEPT_MODULE_EXECUTION)
    } else {
      context
        .runtime_requirements
        .intersects(RuntimeGlobals::REQUIRE_SCOPE | RuntimeGlobals::INTERCEPT_MODULE_EXECUTION)
        || !context
          .runtime_requirements
          .renderable_require_scope()
          .difference(
            RuntimeGlobals::REQUIRE
              | RuntimeGlobals::MODULE_FACTORIES
              | RuntimeGlobals::MODULE_FACTORIES_ADD_ONLY
              | RuntimeGlobals::MODULE_CACHE,
          )
          .is_empty()
    };
  let mut source = ConcatSource::default();

  if module_cache {
    source.add(RawStringSource::from(format!(
      r#"// The module cache
var {} = {{}};
"#,
      context
        .runtime_template
        .render_runtime_variable(&RuntimeVariable::ModuleCache)
    )));
  }

  if require_scope_used {
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

fn render_runtime_global_definition(
  runtime_global: RuntimeGlobals,
  runtime_template: &RuntimeCodeTemplate,
) -> RawStringSource {
  let definition = runtime_template.render_runtime_global_definition(&runtime_global);
  if runtime_global == RuntimeGlobals::MODULE_FACTORIES {
    let modules = runtime_template.render_runtime_variable(&RuntimeVariable::Modules);
    RawStringSource::from(format!(
      "// expose the modules object ({modules})\n{definition} = {modules};\n"
    ))
  } else if runtime_global == RuntimeGlobals::MODULE_CACHE {
    let module_cache = runtime_template.render_runtime_variable(&RuntimeVariable::ModuleCache);
    if runtime_template.render_runtime_globals(&runtime_global) == module_cache {
      RawStringSource::from_static("")
    } else {
      RawStringSource::from(format!(
        "// expose the module cache\n{definition} = {module_cache};\n"
      ))
    }
  } else {
    debug_assert_eq!(runtime_global, RuntimeGlobals::INTERCEPT_MODULE_EXECUTION);
    RawStringSource::from(format!(
      "// expose the module execution interceptor\n{definition} = [];\n"
    ))
  }
}

fn render_runtime_global_definitions(
  context: RuntimeRenderContext<'_>,
  should_export: bool,
) -> ConcatSource {
  let mut source = ConcatSource::default();
  for runtime_global in [
    RuntimeGlobals::MODULE_FACTORIES,
    RuntimeGlobals::MODULE_CACHE,
    RuntimeGlobals::INTERCEPT_MODULE_EXECUTION,
  ] {
    if !context.runtime_requirements.contains(runtime_global) {
      continue;
    }
    source.add(render_runtime_global_definition(
      runtime_global,
      context.runtime_template,
    ));
    if should_export {
      let specifier = context
        .runtime_template
        .render_runtime_globals(&runtime_global);
      source.add(RawStringSource::from(format!(
        "export {{ {specifier} }};\n"
      )));
    }
  }
  source
}
