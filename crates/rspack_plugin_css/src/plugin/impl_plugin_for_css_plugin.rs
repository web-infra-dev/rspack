#![allow(clippy::comparison_chain)]

use std::borrow::Cow;
use std::hash::Hash;
use std::sync::Arc;

use async_trait::async_trait;
use rayon::prelude::*;
use rspack_collections::DatabaseItem;
use rspack_core::rspack_sources::{BoxSource, CachedSource, RawSource, ReplaceSource};
use rspack_core::{
  get_css_chunk_filename_template,
  rspack_sources::{ConcatSource, RawStringSource, Source, SourceExt},
  Chunk, ChunkKind, Module, ModuleType, ParserAndGenerator, PathData, Plugin, RenderManifestEntry,
  SourceType,
};
use rspack_core::{
  AssetInfo, ChunkGraph, ChunkLoading, ChunkLoadingType, ChunkUkey, Compilation,
  CompilationContentHash, CompilationParams, CompilationRenderManifest,
  CompilationRuntimeRequirementInTree, CompilerCompilation, CompilerOptions, DependencyType,
  LibIdentOptions, ModuleGraph, PublicPath, RuntimeGlobals, SelfModuleFactory,
};
use rspack_error::{Diagnostic, Result};
use rspack_hash::RspackHash;
use rspack_hook::plugin_hook;
use rspack_plugin_runtime::is_enabled_for_chunk;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::dependency::{CssLayer, CssMedia, CssSupports};
use crate::parser_and_generator::{CodeGenerationDataUnusedLocalIdent, CssParserAndGenerator};
use crate::runtime::CssLoadingRuntimeModule;
use crate::utils::AUTO_PUBLIC_PATH_PLACEHOLDER;
use crate::{plugin::CssPluginInner, CssPlugin};

struct CssModuleDebugInfo<'a> {
  pub module: &'a dyn Module,
}

impl CssPlugin {
  fn get_chunk_unused_local_idents(
    compilation: &Compilation,
    chunk: &Chunk,
    css_modules: &[&dyn Module],
  ) -> HashSet<String> {
    css_modules
      .iter()
      .filter_map(|module| {
        let module_id = &module.identifier();
        let code_gen_result = compilation
          .code_generation_results
          .get(module_id, Some(chunk.runtime()));
        code_gen_result
          .data
          .get::<CodeGenerationDataUnusedLocalIdent>()
          .map(|data| &data.idents)
      })
      .flat_map(|data| data.iter().cloned())
      .collect()
  }

  fn render_chunk(
    &self,
    compilation: &Compilation,
    mg: &ModuleGraph,
    chunk: &Chunk,
    output_path: &str,
    css_import_modules: Vec<&dyn Module>,
    css_modules: Vec<&dyn Module>,
  ) -> Result<(BoxSource, Vec<Diagnostic>)> {
    let (ordered_css_modules, conflicts) =
      Self::get_ordered_chunk_css_modules(chunk, compilation, css_import_modules, css_modules);
    let source = Self::render_chunk_to_source(compilation, chunk, &ordered_css_modules)?;

    let content = source.source();
    let len = AUTO_PUBLIC_PATH_PLACEHOLDER.len();
    let auto_public_path_matches: Vec<_> = content
      .match_indices(AUTO_PUBLIC_PATH_PLACEHOLDER)
      .map(|(index, _)| (index, index + len))
      .collect();
    let source = if !auto_public_path_matches.is_empty() {
      let mut replace = ReplaceSource::new(source);
      for (start, end) in auto_public_path_matches {
        let relative = PublicPath::render_auto_public_path(compilation, output_path);
        replace.replace(start as u32, end as u32, &relative, None);
      }
      replace.boxed()
    } else {
      source.boxed()
    };
    let mut diagnostics = vec![];
    if let Some(conflicts) = conflicts {
      diagnostics.extend(conflicts.into_iter().map(|conflict| {
        let chunk = compilation.chunk_by_ukey.expect_get(&conflict.chunk);

        let failed_module = mg
          .module_by_identifier(&conflict.failed_module)
          .expect("should have module");
        let selected_module = mg
          .module_by_identifier(&conflict.selected_module)
          .expect("should have module");

        Diagnostic::warn(
          "Conflicting order".into(),
          format!(
            "chunk {}\nConflicting order between {} and {}",
            chunk.name().unwrap_or(
              chunk
                .id(&compilation.chunk_ids_artifact)
                .expect("should have chunk id")
                .as_str()
            ),
            failed_module.readable_identifier(&compilation.options.context),
            selected_module.readable_identifier(&compilation.options.context)
          ),
        )
        .with_file(Some(output_path.to_owned().into()))
        .with_chunk(Some(chunk.ukey().as_u32()))
      }));
    }
    Ok((source, diagnostics))
  }

  fn render_chunk_to_source(
    compilation: &Compilation,
    chunk: &Chunk,
    ordered_css_modules: &[&dyn Module],
  ) -> rspack_error::Result<ConcatSource> {
    let module_sources = ordered_css_modules
      .iter()
      .map(|module| {
        let module_id = &module.identifier();
        let code_gen_result = compilation
          .code_generation_results
          .get(module_id, Some(chunk.runtime()));

        Ok(code_gen_result.get(&SourceType::Css).map(|source| {
          (
            CssModuleDebugInfo { module: *module },
            &code_gen_result.data,
            source,
          )
        }))
      })
      .collect::<Result<Vec<_>>>()?;

    let source = module_sources
      .into_par_iter()
      // TODO(hyf0): I couldn't think of a situation where a module doesn't have `Source`.
      // Should we return a Error if there is a `None` in `module_sources`?
      // Webpack doesn't throw. It just do a best-effort checking https://github.com/webpack/webpack/blob/5e3c4d0ddf8ae6a6e45fea42be4e8950fe49c0bb/lib/css/CssModulesPlugin.js#L565-L568
      .flatten()
      .fold(
        ConcatSource::default,
        |mut acc, (debug_info, data, cur_source)| {
          let (start, end) = Self::render_module_debug_info(compilation, &debug_info);
          acc.add(start);

          let mut num_close_bracket = 0;

          // TODO: use PrefixSource to create indent
          if let Some(media) = data.get::<CssMedia>() {
            num_close_bracket += 1;
            acc.add(RawSource::from(format!("@media {}{{\n", media)));
          }

          if let Some(supports) = data.get::<CssSupports>() {
            num_close_bracket += 1;
            acc.add(RawSource::from(format!("@supports ({}) {{\n", supports)));
          }

          if let Some(layer) = data.get::<CssLayer>() {
            num_close_bracket += 1;
            acc.add(RawSource::from(format!(
              "@layer{} {{\n",
              if let CssLayer::Named(layer) = &layer {
                Cow::Owned(format!(" {}", layer))
              } else {
                Cow::Borrowed("")
              }
            )));
          }

          acc.add(cur_source.clone());

          for _ in 0..num_close_bracket {
            acc.add(RawStringSource::from_static("\n}"));
          }
          acc.add(RawStringSource::from_static("\n"));

          acc.add(end);
          acc
        },
      )
      .reduce(ConcatSource::default, |mut acc, cur| {
        acc.add(cur);
        acc
      });

    Ok(source)
  }

  fn render_module_debug_info(
    compilation: &Compilation,
    debug_info: &CssModuleDebugInfo,
  ) -> (ConcatSource, ConcatSource) {
    let mut start = ConcatSource::default();
    let mut end = ConcatSource::default();

    if !compilation.options.mode.is_development() {
      return (start, end);
    }

    let context = compilation.options.context.as_str();
    let module = debug_info.module;

    let debug_module_id = module
      .lib_ident(LibIdentOptions { context })
      .unwrap_or("None".into());

    start.add(RawStringSource::from(format!(
      "/* #region {:?} */\n",
      debug_module_id,
    )));

    start.add(RawStringSource::from(format!(
      "/*\n- type: {}\n*/\n",
      module.module_type(),
    )));

    end.add(RawStringSource::from(format!(
      "/* #endregion {debug_module_id:?} */\n\n"
    )));

    (start, end)
  }
}

#[plugin_hook(CompilerCompilation for CssPlugin)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  params: &mut CompilationParams,
) -> Result<()> {
  compilation.set_dependency_factory(DependencyType::CssUrl, params.normal_module_factory.clone());
  compilation.set_dependency_factory(
    DependencyType::CssImport,
    params.normal_module_factory.clone(),
  );
  compilation.set_dependency_factory(
    DependencyType::CssCompose,
    params.normal_module_factory.clone(),
  );
  compilation.set_dependency_factory(
    DependencyType::CssSelfReferenceLocalIdent,
    Arc::new(SelfModuleFactory {}),
  );
  Ok(())
}

#[plugin_hook(CompilationRuntimeRequirementInTree for CssPlugin)]
fn runtime_requirements_in_tree(
  &self,
  compilation: &mut Compilation,
  chunk_ukey: &ChunkUkey,
  _all_runtime_requirements: &RuntimeGlobals,
  runtime_requirements: &RuntimeGlobals,
  runtime_requirements_mut: &mut RuntimeGlobals,
) -> Result<Option<()>> {
  let is_enabled_for_chunk = is_enabled_for_chunk(
    chunk_ukey,
    &ChunkLoading::Enable(ChunkLoadingType::Jsonp),
    compilation,
  ) || is_enabled_for_chunk(
    chunk_ukey,
    &ChunkLoading::Enable(ChunkLoadingType::Import),
    compilation,
  );

  if (runtime_requirements.contains(RuntimeGlobals::HAS_CSS_MODULES)
    || runtime_requirements.contains(RuntimeGlobals::ENSURE_CHUNK_HANDLERS)
    || runtime_requirements.contains(RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS))
    && is_enabled_for_chunk
  {
    runtime_requirements_mut.insert(RuntimeGlobals::PUBLIC_PATH);
    runtime_requirements_mut.insert(RuntimeGlobals::GET_CHUNK_CSS_FILENAME);
    runtime_requirements_mut.insert(RuntimeGlobals::HAS_OWN_PROPERTY);
    runtime_requirements_mut.insert(RuntimeGlobals::MODULE_FACTORIES_ADD_ONLY);
    runtime_requirements_mut.insert(RuntimeGlobals::MAKE_NAMESPACE_OBJECT);
    compilation.add_runtime_module(chunk_ukey, Box::<CssLoadingRuntimeModule>::default())?;
  }

  Ok(None)
}

#[plugin_hook(CompilationContentHash for CssPlugin)]
async fn content_hash(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  hashes: &mut HashMap<SourceType, RspackHash>,
) -> Result<()> {
  let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);
  let module_graph = compilation.get_module_graph();
  let css_import_modules = compilation
    .chunk_graph
    .get_chunk_modules_iterable_by_source_type(chunk_ukey, SourceType::CssImport, &module_graph)
    .collect::<Vec<_>>();
  let css_modules = compilation
    .chunk_graph
    .get_chunk_modules_iterable_by_source_type(chunk_ukey, SourceType::Css, &module_graph)
    .collect::<Vec<_>>();
  let (ordered_modules, _) =
    Self::get_ordered_chunk_css_modules(chunk, compilation, css_import_modules, css_modules);
  let mut hasher = hashes
    .entry(SourceType::Css)
    .or_insert_with(|| RspackHash::from(&compilation.options.output));

  ordered_modules
    .iter()
    .map(|m| {
      (
        compilation
          .code_generation_results
          .get_hash(&m.identifier(), Some(chunk.runtime())),
        ChunkGraph::get_module_id(&compilation.module_ids_artifact, m.identifier()),
      )
    })
    .for_each(|(current, id)| {
      if let Some(current) = current {
        current.hash(&mut hasher);
        id.hash(&mut hasher);
      }
    });

  Ok(())
}

#[plugin_hook(CompilationRenderManifest for CssPlugin)]
async fn render_manifest(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  manifest: &mut Vec<RenderManifestEntry>,
  diagnostics: &mut Vec<Diagnostic>,
) -> Result<()> {
  let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);
  if matches!(chunk.kind(), ChunkKind::HotUpdate) {
    return Ok(());
  }
  let module_graph = compilation.get_module_graph();
  let css_import_modules = compilation
    .chunk_graph
    .get_chunk_modules_iterable_by_source_type(chunk_ukey, SourceType::CssImport, &module_graph)
    .collect::<Vec<_>>();
  let css_modules = compilation
    .chunk_graph
    .get_chunk_modules_iterable_by_source_type(chunk_ukey, SourceType::Css, &module_graph)
    .collect::<Vec<_>>();
  if css_import_modules.is_empty() && css_modules.is_empty() {
    return Ok(());
  }

  let filename_template = get_css_chunk_filename_template(
    chunk,
    &compilation.options.output,
    &compilation.chunk_group_by_ukey,
  );
  let mut asset_info = AssetInfo::default();
  let unused_idents = Self::get_chunk_unused_local_idents(compilation, chunk, &css_modules);
  asset_info.set_css_unused_idents(unused_idents);
  let output_path = compilation.get_path_with_info(
    filename_template,
    PathData::default()
      .chunk_id_optional(
        chunk
          .id(&compilation.chunk_ids_artifact)
          .map(|id| id.as_str()),
      )
      .chunk_hash_optional(chunk.rendered_hash(
        &compilation.chunk_hashes_artifact,
        compilation.options.output.hash_digest_length,
      ))
      .chunk_name_optional(chunk.name_for_filename_template(&compilation.chunk_ids_artifact))
      .content_hash_optional(chunk.rendered_content_hash_by_source_type(
        &compilation.chunk_hashes_artifact,
        &SourceType::Css,
        compilation.options.output.hash_digest_length,
      ))
      .runtime(chunk.runtime().as_str()),
    &mut asset_info,
  )?;

  let (source, more_diagnostics) = compilation
    .old_cache
    .chunk_render_occasion
    .use_cache(compilation, chunk, &SourceType::Css, || async {
      let (source, diagnostics) = self.render_chunk(
        compilation,
        &module_graph,
        chunk,
        &output_path,
        css_import_modules,
        css_modules,
      )?;
      Ok((CachedSource::new(source).boxed(), diagnostics))
    })
    .await?;

  diagnostics.extend(more_diagnostics);
  manifest.push(RenderManifestEntry {
    source: source.boxed(),
    filename: output_path,
    has_filename: false,
    info: asset_info,
    auxiliary: false,
  });
  Ok(())
}

#[async_trait]
impl Plugin for CssPlugin {
  fn name(&self) -> &'static str {
    "css"
  }

  fn apply(
    &self,
    ctx: rspack_core::PluginContext<&mut rspack_core::ApplyContext>,
    _options: &CompilerOptions,
  ) -> Result<()> {
    ctx
      .context
      .compiler_hooks
      .compilation
      .tap(compilation::new(self));
    ctx
      .context
      .compilation_hooks
      .runtime_requirement_in_tree
      .tap(runtime_requirements_in_tree::new(self));
    ctx
      .context
      .compilation_hooks
      .content_hash
      .tap(content_hash::new(self));
    ctx
      .context
      .compilation_hooks
      .render_manifest
      .tap(render_manifest::new(self));

    ctx.context.register_parser_and_generator_builder(
      ModuleType::Css,
      Box::new(|p, g| {
        let p = p
          .and_then(|p| p.get_css())
          .expect("should have CssParserOptions");
        let g = g
          .and_then(|g| g.get_css())
          .expect("should have CssGeneratorOptions");
        Box::new(CssParserAndGenerator {
          exports: None,
          local_names: None,
          convention: None,
          local_ident_name: None,
          exports_only: g.exports_only.expect("should have exports_only"),
          named_exports: p.named_exports.expect("should have named_exports"),
          es_module: g.es_module.expect("should have es_module"),
          hot: false,
        }) as Box<dyn ParserAndGenerator>
      }),
    );
    ctx.context.register_parser_and_generator_builder(
      ModuleType::CssModule,
      Box::new(|p, g| {
        let p = p
          .and_then(|p| p.get_css_module())
          .expect("should have CssModuleParserOptions");
        let g = g
          .and_then(|g| g.get_css_module())
          .expect("should have CssModuleGeneratorOptions");
        Box::new(CssParserAndGenerator {
          exports: None,
          local_names: None,
          convention: Some(
            g.exports_convention
              .expect("should have exports_convention"),
          ),
          local_ident_name: Some(
            g.local_ident_name
              .clone()
              .expect("should have local_ident_name"),
          ),
          exports_only: g.exports_only.expect("should have exports_only"),
          named_exports: p.named_exports.expect("should have named_exports"),
          es_module: g.es_module.expect("should have es_module"),
          hot: false,
        }) as Box<dyn ParserAndGenerator>
      }),
    );
    ctx.context.register_parser_and_generator_builder(
      ModuleType::CssAuto,
      Box::new(|p, g| {
        let p = p
          .and_then(|p| p.get_css_auto())
          .expect("should have CssAutoParserOptions");
        let g = g
          .and_then(|g| g.get_css_auto())
          .expect("should have CssAutoGeneratorOptions");
        Box::new(CssParserAndGenerator {
          exports: None,
          local_names: None,
          convention: Some(
            g.exports_convention
              .expect("should have exports_convention"),
          ),
          local_ident_name: Some(
            g.local_ident_name
              .clone()
              .expect("should have local_ident_name"),
          ),
          exports_only: g.exports_only.expect("should have exports_only"),
          named_exports: p.named_exports.expect("should have named_exports"),
          es_module: g.es_module.expect("should have es_module"),
          hot: false,
        }) as Box<dyn ParserAndGenerator>
      }),
    );

    Ok(())
  }
}
