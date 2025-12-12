use std::hash::Hash;

use rspack_core::{
  ChunkGraph, ChunkKind, ChunkUkey, Compilation, CompilationAdditionalChunkRuntimeRequirements,
  CompilationDependentFullHash, CompilationParams, CompilerCompilation, ModuleIdentifier, Plugin,
  RuntimeGlobals, RuntimeVariable, SourceType,
  rspack_sources::{ConcatSource, RawStringSource, Source, SourceExt},
};
use rspack_error::Result;
use rspack_hash::RspackHash;
use rspack_hook::{plugin, plugin_hook};
use rspack_plugin_javascript::{
  JavascriptModulesChunkHash, JavascriptModulesRenderChunk, JavascriptModulesRenderStartup,
  JsPlugin, RenderSource, runtime::render_chunk_runtime_modules,
};
use rspack_util::{itoa, json_stringify};
use rustc_hash::FxHashSet as HashSet;

use super::update_hash_for_entry_startup;
use crate::{
  chunk_has_js, get_all_chunks, get_chunk_output_name, get_relative_path,
  get_runtime_chunk_output_name, runtime_chunk_has_hash,
};

const PLUGIN_NAME: &str = "rspack.ModuleChunkFormatPlugin";

#[plugin]
#[derive(Debug, Default)]
pub struct ModuleChunkFormatPlugin;

#[plugin_hook(CompilerCompilation for ModuleChunkFormatPlugin)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  let hooks = JsPlugin::get_compilation_hooks_mut(compilation.id());
  let mut hooks = hooks.write().await;
  hooks.render_chunk.tap(render_chunk::new(self));
  hooks.render_startup.tap(render_startup::new(self));
  hooks.chunk_hash.tap(js_chunk_hash::new(self));
  Ok(())
}

#[plugin_hook(CompilationAdditionalChunkRuntimeRequirements for ModuleChunkFormatPlugin)]
async fn additional_chunk_runtime_requirements(
  &self,
  compilation: &mut Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_requirements: &mut RuntimeGlobals,
) -> Result<()> {
  let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);

  if chunk.has_runtime(&compilation.chunk_group_by_ukey) {
    return Ok(());
  }

  if compilation
    .chunk_graph
    .get_number_of_entry_modules(chunk_ukey)
    > 0
  {
    runtime_requirements.insert(RuntimeGlobals::REQUIRE);
    runtime_requirements.insert(RuntimeGlobals::EXTERNAL_INSTALL_CHUNK);
  }

  Ok(())
}

#[plugin_hook(JavascriptModulesChunkHash for ModuleChunkFormatPlugin)]
async fn js_chunk_hash(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  hasher: &mut RspackHash,
) -> Result<()> {
  let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);
  if chunk.has_runtime(&compilation.chunk_group_by_ukey) {
    return Ok(());
  }

  PLUGIN_NAME.hash(hasher);

  update_hash_for_entry_startup(
    hasher,
    compilation,
    compilation
      .chunk_graph
      .get_chunk_entry_modules_with_chunk_group_iterable(chunk_ukey),
    chunk_ukey,
  );

  Ok(())
}

#[plugin_hook(CompilationDependentFullHash for ModuleChunkFormatPlugin)]
async fn compilation_dependent_full_hash(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
) -> Result<Option<bool>> {
  if !chunk_has_js(chunk_ukey, compilation) {
    return Ok(None);
  }

  let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);

  if !chunk.has_entry_module(&compilation.chunk_graph) {
    return Ok(None);
  }

  if runtime_chunk_has_hash(compilation, chunk_ukey).await? {
    return Ok(Some(true));
  }

  Ok(None)
}

#[plugin_hook(JavascriptModulesRenderChunk for ModuleChunkFormatPlugin)]
async fn render_chunk(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  render_source: &mut RenderSource,
) -> Result<()> {
  let hooks = JsPlugin::get_compilation_hooks(compilation.id());
  let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);
  let base_chunk_output_name = get_chunk_output_name(chunk, compilation).await?;

  let chunk_id_json_string = json_stringify(chunk.expect_id());

  let mut sources = ConcatSource::default();
  sources.add(RawStringSource::from(format!(
    "export const __rspack_esm_id = {chunk_id_json_string};\n",
  )));
  sources.add(RawStringSource::from(format!(
    "export const __rspack_esm_ids = [{chunk_id_json_string}];\n",
  )));
  sources.add(RawStringSource::from(format!(
    "export const {} = ",
    compilation
      .runtime_template
      .render_runtime_variable(&RuntimeVariable::Modules),
  )));
  sources.add(render_source.source.clone());
  sources.add(RawStringSource::from_static(";\n"));

  if compilation
    .chunk_graph
    .has_chunk_runtime_modules(chunk_ukey)
  {
    sources.add(RawStringSource::from_static(
      "export const __rspack_esm_runtime = ",
    ));
    sources.add(render_chunk_runtime_modules(compilation, chunk_ukey).await?);
    sources.add(RawStringSource::from_static(";\n"));
  }

  if matches!(chunk.kind(), ChunkKind::HotUpdate) {
    render_source.source = sources.boxed();
    return Ok(());
  }

  if chunk.has_entry_module(&compilation.chunk_graph) {
    let runtime_chunk_output_name = get_runtime_chunk_output_name(compilation, chunk_ukey).await?;
    sources.add(RawStringSource::from(format!(
      "import {{ {} }} from '{}';\n",
      compilation
        .runtime_template
        .render_runtime_globals(&RuntimeGlobals::REQUIRE),
      get_relative_path(
        base_chunk_output_name
          .trim_start_matches("/")
          .trim_start_matches("\\"),
        &runtime_chunk_output_name
      )
    )));

    let entries = compilation
      .chunk_graph
      .get_chunk_entry_modules_with_chunk_group_iterable(chunk_ukey);

    let mut startup_source = vec![];

    startup_source.push(format!(
      "var {} = function(moduleId) {{ return {}({} = moduleId); }}",
      compilation
        .runtime_template
        .render_runtime_variable(&RuntimeVariable::StartupExec),
      compilation
        .runtime_template
        .render_runtime_globals(&RuntimeGlobals::REQUIRE),
      compilation
        .runtime_template
        .render_runtime_globals(&RuntimeGlobals::ENTRY_MODULE_ID)
    ));

    let module_graph = compilation.get_module_graph();
    let mut loaded_chunks = HashSet::default();
    for (i, (module, entry)) in entries.iter().enumerate() {
      if !module_graph
        .module_by_identifier(module)
        .is_some_and(|module| {
          module
            .source_types(&module_graph)
            .contains(&SourceType::JavaScript)
        })
      {
        continue;
      }
      let module_id = ChunkGraph::get_module_id(&compilation.module_ids_artifact, *module)
        .expect("should have module id");
      let runtime_chunk = compilation
        .chunk_group_by_ukey
        .expect_get(entry)
        .get_runtime_chunk(&compilation.chunk_group_by_ukey);
      let chunks = get_all_chunks(
        entry,
        &runtime_chunk,
        None,
        &compilation.chunk_group_by_ukey,
      );

      for chunk_ukey in chunks.iter() {
        // Skip processing if the chunk doesn't have any JavaScript
        if !chunk_has_js(chunk_ukey, compilation) {
          continue;
        }
        if loaded_chunks.contains(chunk_ukey) {
          continue;
        }
        loaded_chunks.insert(*chunk_ukey);
        let index = loaded_chunks.len();
        let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);
        let other_chunk_output_name = get_chunk_output_name(chunk, compilation).await?;
        let mut index_buffer = itoa::Buffer::new();
        let index_str = index_buffer.format(index);
        startup_source.push(format!(
          "import * as __rspack_chunk_{} from '{}';",
          index_str,
          get_relative_path(&base_chunk_output_name, &other_chunk_output_name)
        ));
        let mut index_buffer2 = itoa::Buffer::new();
        let index_str2 = index_buffer2.format(index);
        startup_source.push(format!(
          "{}(__rspack_chunk_{});",
          compilation
            .runtime_template
            .render_runtime_globals(&RuntimeGlobals::EXTERNAL_INSTALL_CHUNK),
          index_str2
        ));
      }

      let module_id_expr = serde_json::to_string(module_id).expect("invalid module_id");

      startup_source.push(format!(
        "{}{}({module_id_expr});",
        if i + 1 == entries.len() {
          format!(
            "var {} = ",
            compilation
              .runtime_template
              .render_runtime_variable(&RuntimeVariable::Exports)
          )
        } else {
          "".to_string()
        },
        compilation
          .runtime_template
          .render_runtime_variable(&RuntimeVariable::StartupExec),
      ));
    }

    let last_entry_module = entries
      .keys()
      .next_back()
      .expect("should have last entry module");
    let mut render_source = RenderSource {
      source: RawStringSource::from(startup_source.join("\n")).boxed(),
    };
    hooks
      .try_read()
      .expect("should have js plugin drive")
      .render_startup
      .call(
        compilation,
        chunk_ukey,
        last_entry_module,
        &mut render_source,
      )
      .await?;
    sources.add(render_source.source);
  }
  render_source.source = sources.boxed();
  Ok(())
}

fn render_chunk_import(named_import: &str, import_source: &str) -> String {
  format!("import * as {} from '{}';\n", named_import, import_source)
}
#[plugin_hook(JavascriptModulesRenderStartup for ModuleChunkFormatPlugin)]
async fn render_startup(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  _module: &ModuleIdentifier,
  render_source: &mut RenderSource,
) -> Result<()> {
  let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);
  let entries_count = compilation
    .chunk_graph
    .get_number_of_entry_modules(chunk_ukey);
  let has_runtime = chunk.has_runtime(&compilation.chunk_group_by_ukey);

  if entries_count > 0 && has_runtime {
    let dependent_chunks = compilation
      .chunk_graph
      .get_chunk_entry_dependent_chunks_iterable(
        chunk_ukey,
        &compilation.chunk_by_ukey,
        &compilation.chunk_group_by_ukey,
      );

    let base_chunk_output_name = get_chunk_output_name(chunk, compilation).await?;

    let mut dependent_load = ConcatSource::default();
    for (index, ck) in dependent_chunks.enumerate() {
      if !chunk_has_js(&ck, compilation) {
        continue;
      }

      let dependant_chunk = compilation.chunk_by_ukey.expect_get(&ck);

      let named_import = format!("__rspack_imports_{}", index);

      let dependant_chunk_name = get_chunk_output_name(dependant_chunk, compilation).await?;

      let imported = get_relative_path(&base_chunk_output_name, &dependant_chunk_name);

      dependent_load.add(RawStringSource::from(render_chunk_import(
        &named_import,
        &imported,
      )));
      dependent_load.add(RawStringSource::from(format!(
        "{}({});\n",
        compilation
          .runtime_template
          .render_runtime_globals(&RuntimeGlobals::EXTERNAL_INSTALL_CHUNK),
        named_import
      )));
    }

    if !dependent_load.source().is_empty() {
      let mut sources = ConcatSource::default();
      sources.add(dependent_load);
      sources.add(render_source.source.clone());
      render_source.source = sources.boxed();
    }
  }
  Ok(())
}

impl Plugin for ModuleChunkFormatPlugin {
  fn name(&self) -> &'static str {
    PLUGIN_NAME
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compiler_hooks.compilation.tap(compilation::new(self));
    ctx
      .compilation_hooks
      .additional_chunk_runtime_requirements
      .tap(additional_chunk_runtime_requirements::new(self));
    ctx
      .compilation_hooks
      .dependent_full_hash
      .tap(compilation_dependent_full_hash::new(self));
    Ok(())
  }
}
