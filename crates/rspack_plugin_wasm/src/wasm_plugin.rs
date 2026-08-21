use std::fmt::Debug;

use rspack_core::{
  ChunkUkey, Compilation, CompilationParams, CompilationRenderManifest, CompilerCompilation,
  DependencyType, ManifestAssetType, ModuleType, ParserAndGenerator, Plugin, RenderManifestEntry,
  SourceType,
};
use rspack_error::{Diagnostic, Result};
use rspack_hook::{plugin, plugin_hook};

use crate::parser_and_generator::{AsyncWasmParserAndGenerator, CodeGenerationDataWasmFilename};

#[plugin]
#[derive(Debug, Default)]
pub struct AsyncWasmPlugin {}

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
  let chunk = compilation
    .build_chunk_graph_artifact
    .chunk_by_ukey
    .expect_get(chunk_ukey);
  let module_graph = &compilation.get_module_graph();

  let ordered_modules = compilation
    .build_chunk_graph_artifact
    .chunk_graph
    .get_chunk_modules(chunk_ukey, module_graph);

  for m in ordered_modules {
    if m.module_type() != &ModuleType::WasmAsync {
      continue;
    }
    let Some(source) = compilation
      .code_generation_results
      .get(&m.identifier(), Some(chunk.runtime()))
      .get(&SourceType::Wasm)
    else {
      continue;
    };

    let filename_data = compilation
      .code_generation_results
      .get(&m.identifier(), Some(chunk.runtime()))
      .data
      .get::<CodeGenerationDataWasmFilename>()
      .expect("should have filename for async wasm module");
    let asset_info = filename_data
      .asset_info
      .clone()
      .with_asset_type(ManifestAssetType::Wasm);
    manifest.push(RenderManifestEntry {
      source: source.clone(),
      filename: filename_data.filename.clone(),
      has_filename: true,
      info: asset_info,
      auxiliary: false,
    })
  }

  Ok(())
}

impl Plugin for AsyncWasmPlugin {
  fn name(&self) -> &'static str {
    "rspack.AsyncWebAssemblyModulesPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compiler_hooks.compilation.tap(compilation::new(self));
    ctx
      .compilation_hooks
      .render_manifest
      .tap(render_manifest::new(self));

    ctx.register_parser_and_generator_builder(
      ModuleType::WasmAsync,
      Box::new(move |_| Box::new(AsyncWasmParserAndGenerator) as Box<dyn ParserAndGenerator>),
    );

    Ok(())
  }
}
