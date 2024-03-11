use std::hash::Hash;

use async_trait::async_trait;
use rspack_core::rspack_sources::{ConcatSource, RawSource, SourceExt};
use rspack_core::{
  AdditionalChunkRuntimeRequirementsArgs, ChunkKind, JsChunkHashArgs, Plugin,
  PluginAdditionalChunkRuntimeRequirementsOutput, PluginContext, PluginJsChunkHashHookOutput,
  PluginRenderChunkHookOutput, RenderChunkArgs, RenderStartupArgs, RuntimeGlobals,
};
use rspack_plugin_javascript::runtime::render_chunk_runtime_modules;
use rustc_hash::FxHashSet as HashSet;

use super::update_hash_for_entry_startup;
use crate::{
  get_all_chunks, get_chunk_output_name, get_relative_path, get_runtime_chunk_output_name,
};

#[derive(Debug)]
pub struct ModuleChunkFormatPlugin;

#[async_trait]
impl Plugin for ModuleChunkFormatPlugin {
  fn name(&self) -> &'static str {
    "ModuleChunkFormatPlugin"
  }

  async fn additional_chunk_runtime_requirements(
    &self,
    _ctx: PluginContext,
    args: &mut AdditionalChunkRuntimeRequirementsArgs,
  ) -> PluginAdditionalChunkRuntimeRequirementsOutput {
    let compilation = &mut args.compilation;
    let chunk_ukey = args.chunk;
    let runtime_requirements = &mut args.runtime_requirements;
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
      runtime_requirements.insert(RuntimeGlobals::STARTUP_ENTRYPOINT);
      runtime_requirements.insert(RuntimeGlobals::EXTERNAL_INSTALL_CHUNK);
    }

    Ok(())
  }

  fn js_chunk_hash(
    &self,
    _ctx: PluginContext,
    args: &mut JsChunkHashArgs,
  ) -> PluginJsChunkHashHookOutput {
    if args
      .chunk()
      .has_runtime(&args.compilation.chunk_group_by_ukey)
    {
      return Ok(());
    }

    self.name().hash(&mut args.hasher);

    update_hash_for_entry_startup(
      args.hasher,
      args.compilation,
      args
        .compilation
        .chunk_graph
        .get_chunk_entry_modules_with_chunk_group_iterable(args.chunk_ukey),
      args.chunk_ukey,
    );

    Ok(())
  }

  async fn render_chunk(
    &self,
    _ctx: PluginContext,
    args: &RenderChunkArgs,
  ) -> PluginRenderChunkHookOutput {
    let compilation = args.compilation;
    let chunk = args.chunk();
    let base_chunk_output_name = get_chunk_output_name(chunk, compilation);
    if matches!(chunk.kind, ChunkKind::HotUpdate) {
      unreachable!("HMR is not implemented for module chunk format yet");
    }

    let mut sources = ConcatSource::default();
    sources.add(RawSource::from(format!(
      "export const ids = ['{}'];\n",
      &chunk.expect_id().to_string()
    )));
    sources.add(RawSource::from("export const modules = "));
    sources.add(args.module_source.clone());
    sources.add(RawSource::from(";\n"));

    if compilation
      .chunk_graph
      .has_chunk_runtime_modules(args.chunk_ukey)
    {
      sources.add(RawSource::from("export const runtime = "));
      sources.add(render_chunk_runtime_modules(compilation, args.chunk_ukey)?);
      sources.add(RawSource::from(";\n"));
    }

    if chunk.has_entry_module(&compilation.chunk_graph) {
      let runtime_chunk_output_name = get_runtime_chunk_output_name(args)?;
      sources.add(RawSource::from(format!(
        "import __webpack_require__ from '{}';\n",
        get_relative_path(&base_chunk_output_name, &runtime_chunk_output_name)
      )));

      let entries = compilation
        .chunk_graph
        .get_chunk_entry_modules_with_chunk_group_iterable(args.chunk_ukey);

      let mut startup_source = vec![];

      startup_source.push(format!(
        "var __webpack_exec__ = function(moduleId) {{ return __webpack_require__({} = moduleId); }}",
        RuntimeGlobals::ENTRY_MODULE_ID
      ));

      let mut loaded_chunks = HashSet::default();
      for (i, (module, entry)) in entries.iter().enumerate() {
        let module_id = compilation
          .get_module_graph()
          .module_graph_module_by_identifier(module)
          .map(|module| module.id(&compilation.chunk_graph))
          .expect("should have module id");
        let runtime_chunk = compilation
          .chunk_group_by_ukey
          .expect_get(entry)
          .get_runtime_chunk();
        let chunks = get_all_chunks(
          entry,
          &runtime_chunk,
          None,
          &compilation.chunk_group_by_ukey,
        );

        for chunk_ukey in chunks.iter() {
          if loaded_chunks.contains(chunk_ukey) {
            continue;
          }
          loaded_chunks.insert(*chunk_ukey);
          let index = loaded_chunks.len();
          let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);
          let other_chunk_output_name = get_chunk_output_name(chunk, compilation);
          startup_source.push(format!(
            "import * as __webpack_chunk_${index}__ from '{}';",
            get_relative_path(&base_chunk_output_name, &other_chunk_output_name)
          ));
          startup_source.push(format!(
            "{}(__webpack_chunk_${index}__);",
            RuntimeGlobals::EXTERNAL_INSTALL_CHUNK
          ));
        }

        let module_id_expr = serde_json::to_string(module_id).expect("invalid module_id");

        startup_source.push(format!(
          "{}__webpack_exec__({module_id_expr});",
          if i + 1 == entries.len() {
            "var __webpack_exports__ = "
          } else {
            ""
          }
        ));
      }

      let last_entry_module = entries
        .keys()
        .last()
        .expect("should have last entry module");
      if let Some(s) = compilation
        .plugin_driver
        .render_startup(RenderStartupArgs {
          compilation,
          chunk: &chunk.ukey,
          module: *last_entry_module,
          source: RawSource::from(startup_source.join("\n")).boxed(),
        })?
      {
        sources.add(s);
      }
    }

    Ok(Some(sources.boxed()))
  }
}
