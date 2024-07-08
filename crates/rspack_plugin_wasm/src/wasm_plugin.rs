use std::fmt::Debug;

use async_trait::async_trait;
use rayon::prelude::*;
use rspack_core::{
  ApplyContext, ChunkUkey, Compilation, CompilationParams, CompilationRenderManifest,
  CompilerCompilation, CompilerOptions, DependencyType, ModuleType, ParserAndGenerator, Plugin,
  PluginContext, RenderManifestEntry, SourceType,
};
use rspack_error::{Diagnostic, Result};
use rspack_hook::{plugin, plugin_hook};

use crate::{AsyncWasmParserAndGenerator, ModuleIdToFileName};

pub struct EnableWasmLoadingPlugin;

#[plugin]
#[derive(Debug, Default)]
pub struct AsyncWasmPlugin {
  pub module_id_to_filename_without_ext: ModuleIdToFileName,
}

#[plugin_hook(CompilerCompilation for AsyncWasmPlugin)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  params: &mut CompilationParams,
) -> Result<()> {
  compilation.set_dependency_factory(
    DependencyType::WasmImport,
    params.normal_module_factory.clone(),
  );
  compilation.set_dependency_factory(
    DependencyType::WasmExportImported,
    params.normal_module_factory.clone(),
  );
  Ok(())
}

#[plugin_hook(CompilationRenderManifest for AsyncWasmPlugin)]
async fn render_manifest(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  manifest: &mut Vec<RenderManifestEntry>,
  _diagnostics: &mut Vec<Diagnostic>,
) -> Result<()> {
  let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);
  let module_graph = &compilation.get_module_graph();

  let ordered_modules = compilation
    .chunk_graph
    .get_chunk_modules(chunk_ukey, module_graph);

  let files = ordered_modules
    .par_iter()
    .filter(|m| *m.module_type() == ModuleType::WasmAsync)
    .map(|m| {
      let code_gen_result = compilation
        .code_generation_results
        .get(&m.identifier(), Some(&chunk.runtime));

      let result = code_gen_result.get(&SourceType::Wasm).map(|source| {
        let (output_path, asset_info) = self
          .module_id_to_filename_without_ext
          .get(&m.identifier())
          .map(|s| s.clone())
          .expect("should have wasm_filename");
        RenderManifestEntry::new(source.clone(), output_path, asset_info, false, false)
      });

      Ok(result)
    })
    .collect::<Result<Vec<Option<RenderManifestEntry>>>>()?
    .into_iter()
    .flatten()
    .collect::<Vec<RenderManifestEntry>>();
  manifest.extend(files);

  Ok(())
}

#[async_trait]
impl Plugin for AsyncWasmPlugin {
  fn name(&self) -> &'static str {
    "rspack.AsyncWebAssemblyModulesPlugin"
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
      .compilation_hooks
      .render_manifest
      .tap(render_manifest::new(self));

    let module_id_to_filename_without_ext = self.module_id_to_filename_without_ext.clone();

    ctx.context.register_parser_and_generator_builder(
      ModuleType::WasmAsync,
      Box::new(move |_, _| {
        Box::new({
          AsyncWasmParserAndGenerator {
            module_id_to_filename: module_id_to_filename_without_ext.clone(),
          }
        }) as Box<dyn ParserAndGenerator>
      }),
    );

    Ok(())
  }
}
