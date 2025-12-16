#![allow(clippy::comparison_chain)]

use std::{
  borrow::Cow,
  hash::Hash,
  sync::{Arc, LazyLock},
};

use atomic_refcell::AtomicRefCell;
use rspack_collections::{DatabaseItem, ItemUkey};
use rspack_core::{
  AssetInfo, Chunk, ChunkGraph, ChunkKind, ChunkLoading, ChunkLoadingType, ChunkUkey, Compilation,
  CompilationContentHash, CompilationId, CompilationParams, CompilationRenderManifest,
  CompilationRuntimeRequirementInTree, CompilerCompilation, DependencyType, ManifestAssetType,
  Module, ModuleGraph, ModuleType, ParserAndGenerator, PathData, Plugin, PublicPath,
  RenderManifestEntry, RuntimeGlobals, RuntimeModuleExt, SelfModuleFactory, SourceType,
  get_css_chunk_filename_template,
  rspack_sources::{
    BoxSource, CachedSource, ConcatSource, RawStringSource, ReplaceSource, Source, SourceExt,
  },
};
use rspack_error::{Diagnostic, Result, ToStringResultToRspackResultExt};
use rspack_hash::RspackHash;
use rspack_hook::plugin_hook;
use rspack_plugin_runtime::is_enabled_for_chunk;
use rspack_util::fx_hash::FxDashMap;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::{
  CssPlugin,
  dependency::{
    CssImportDependencyTemplate, CssLayer, CssLocalIdentDependencyTemplate, CssMedia,
    CssSelfReferenceLocalIdentDependencyTemplate, CssSupports, CssUrlDependencyTemplate,
  },
  parser_and_generator::{CodeGenerationDataUnusedLocalIdent, CssParserAndGenerator},
  plugin::{CssModulesPluginHooks, CssModulesRenderSource, CssPluginInner},
  runtime::CssLoadingRuntimeModule,
  utils::AUTO_PUBLIC_PATH_PLACEHOLDER,
};

/// Safety with [atomic_refcell::AtomicRefCell]:
///
/// We should make sure that there's no read-write and write-write conflicts for each hook instance by looking up [CssPlugin::get_compilation_hooks_mut]
type ArcCssModulesPluginHooks = Arc<AtomicRefCell<CssModulesPluginHooks>>;

static COMPILATION_HOOKS_MAP: LazyLock<FxDashMap<CompilationId, ArcCssModulesPluginHooks>> =
  LazyLock::new(Default::default);

struct CssModuleDebugInfo<'a> {
  pub module: &'a dyn Module,
}

impl CssPlugin {
  pub fn get_compilation_hooks(id: CompilationId) -> ArcCssModulesPluginHooks {
    if !COMPILATION_HOOKS_MAP.contains_key(&id) {
      COMPILATION_HOOKS_MAP.insert(id, Default::default());
    }
    COMPILATION_HOOKS_MAP
      .get(&id)
      .expect("should have js plugin drive")
      .clone()
  }

  pub fn get_compilation_hooks_mut(id: CompilationId) -> ArcCssModulesPluginHooks {
    COMPILATION_HOOKS_MAP.entry(id).or_default().clone()
  }

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

  async fn render_chunk(
    &self,
    compilation: &Compilation,
    mg: &ModuleGraph,
    chunk: &Chunk,
    output_path: &str,
    css_import_modules: Vec<&dyn Module>,
    css_modules: Vec<&dyn Module>,
  ) -> Result<(BoxSource, Vec<Diagnostic>)> {
    let css_plugin_hooks = Self::get_compilation_hooks(compilation.id());
    let hooks = css_plugin_hooks.borrow();
    let (ordered_css_modules, conflicts) =
      Self::get_ordered_chunk_css_modules(chunk, compilation, css_import_modules, css_modules);
    let source =
      Self::render_chunk_to_source(compilation, chunk, &ordered_css_modules, &hooks).await?;

    let content = source.source().into_string_lossy();
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

        let mut diagnostic = Diagnostic::warn(
          "Conflicting order".into(),
          format!(
            "chunk {}\nConflicting order between {} and {}",
            chunk
              .name()
              .unwrap_or(chunk.id().expect("should have chunk id").as_str()),
            failed_module.readable_identifier(&compilation.options.context),
            selected_module.readable_identifier(&compilation.options.context)
          ),
        );
        diagnostic.file = Some(output_path.to_owned().into());
        diagnostic.chunk = Some(chunk.ukey().as_u32());
        diagnostic
      }));
    }
    Ok((source, diagnostics))
  }

  async fn render_chunk_to_source(
    compilation: &Compilation,
    chunk: &Chunk,
    ordered_css_modules: &[&dyn Module],
    hooks: &CssModulesPluginHooks,
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

    let module_sources = rspack_futures::scope::<_, Result<_>>(|token| {
      module_sources
        .into_iter()
        .flatten()
        .for_each(|(debug_info, data, cur_source)| {
          let s = unsafe {
            token.used((
              compilation,
              chunk.ukey(),
              debug_info,
              data,
              cur_source,
              hooks,
            ))
          };
          s.spawn(
            |(compilation, chunk, debug_info, data, cur_source, hooks)| async move {
              let mut post_module_container = {
                let mut container_source = ConcatSource::default();

                let mut num_close_bracket = 0;

                // TODO: use PrefixSource to create indent
                if let Some(media) = data.get::<CssMedia>() {
                  num_close_bracket += 1;
                  container_source.add(RawStringSource::from(format!("@media {media}{{\n")));
                }

                if let Some(supports) = data.get::<CssSupports>() {
                  num_close_bracket += 1;
                  container_source.add(RawStringSource::from(format!(
                    "@supports ({supports}) {{\n"
                  )));
                }

                if let Some(layer) = data.get::<CssLayer>() {
                  num_close_bracket += 1;
                  container_source.add(RawStringSource::from(format!(
                    "@layer{} {{\n",
                    if let CssLayer::Named(layer) = &layer {
                      Cow::Owned(format!(" {layer}"))
                    } else {
                      Cow::Borrowed("")
                    }
                  )));
                }

                container_source.add(cur_source.clone());

                for _ in 0..num_close_bracket {
                  container_source.add(RawStringSource::from_static("\n}"));
                }
                container_source.add(RawStringSource::from_static("\n"));
                CssModulesRenderSource {
                  source: container_source.boxed(),
                }
              };

              let chunk_ukey = chunk.ukey().as_u32().into();
              hooks
                .render_module_package
                .call(
                  compilation,
                  &chunk_ukey,
                  debug_info.module,
                  &mut post_module_container,
                )
                .await?;

              Ok(post_module_container.source)
            },
          );
        });
    })
    .await
    .into_iter()
    .map(|r| r.to_rspack_result())
    .collect::<Result<Vec<_>>>()?;

    let mut source = ConcatSource::default();

    for module_source in module_sources.into_iter() {
      source.add(module_source?);
    }

    Ok(source)
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
  compilation.set_dependency_template(
    CssImportDependencyTemplate::template_type(),
    Arc::new(CssImportDependencyTemplate::default()),
  );
  compilation.set_dependency_template(
    CssLocalIdentDependencyTemplate::template_type(),
    Arc::new(CssLocalIdentDependencyTemplate::default()),
  );
  compilation.set_dependency_template(
    CssSelfReferenceLocalIdentDependencyTemplate::template_type(),
    Arc::new(CssSelfReferenceLocalIdentDependencyTemplate::default()),
  );
  compilation.set_dependency_template(
    CssUrlDependencyTemplate::template_type(),
    Arc::new(CssUrlDependencyTemplate::default()),
  );
  Ok(())
}

#[plugin_hook(CompilationRuntimeRequirementInTree for CssPlugin)]
async fn runtime_requirements_in_tree(
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
    compilation.add_runtime_module(
      chunk_ukey,
      CssLoadingRuntimeModule::new(&compilation.runtime_template).boxed(),
    )?;
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
  let css_import_modules = compilation.chunk_graph.get_chunk_modules_by_source_type(
    chunk_ukey,
    SourceType::CssImport,
    &module_graph,
  );
  let css_modules = compilation.chunk_graph.get_chunk_modules_by_source_type(
    chunk_ukey,
    SourceType::Css,
    &module_graph,
  );
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
  let css_import_modules = compilation.chunk_graph.get_chunk_modules_by_source_type(
    chunk_ukey,
    SourceType::CssImport,
    &module_graph,
  );
  let css_modules = compilation.chunk_graph.get_chunk_modules_by_source_type(
    chunk_ukey,
    SourceType::Css,
    &module_graph,
  );
  if css_import_modules.is_empty() && css_modules.is_empty() {
    return Ok(());
  }

  let filename_template = get_css_chunk_filename_template(
    chunk,
    &compilation.options.output,
    &compilation.chunk_group_by_ukey,
  );
  let mut asset_info = AssetInfo::default().with_asset_type(ManifestAssetType::Css);
  let unused_idents = Self::get_chunk_unused_local_idents(compilation, chunk, &css_modules);
  asset_info.set_css_unused_idents(unused_idents);
  let output_path = compilation
    .get_path_with_info(
      filename_template,
      PathData::default()
        .chunk_id_optional(chunk.id().map(|id| id.as_str()))
        .chunk_hash_optional(chunk.rendered_hash(
          &compilation.chunk_hashes_artifact,
          compilation.options.output.hash_digest_length,
        ))
        .chunk_name_optional(chunk.name_for_filename_template())
        .content_hash_optional(chunk.rendered_content_hash_by_source_type(
          &compilation.chunk_hashes_artifact,
          &SourceType::Css,
          compilation.options.output.hash_digest_length,
        ))
        .runtime(chunk.runtime().as_str()),
      &mut asset_info,
    )
    .await?;

  let (source, more_diagnostics) = compilation
    .chunk_render_cache_artifact
    .use_cache(compilation, chunk, &SourceType::Css, || async {
      let (source, diagnostics) = self
        .render_chunk(
          compilation,
          &module_graph,
          chunk,
          &output_path,
          css_import_modules,
          css_modules,
        )
        .await?;
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

impl Plugin for CssPlugin {
  fn name(&self) -> &'static str {
    "css"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compiler_hooks.compilation.tap(compilation::new(self));
    ctx
      .compilation_hooks
      .runtime_requirement_in_tree
      .tap(runtime_requirements_in_tree::new(self));
    ctx
      .compilation_hooks
      .content_hash
      .tap(content_hash::new(self));
    ctx
      .compilation_hooks
      .render_manifest
      .tap(render_manifest::new(self));

    ctx.register_parser_and_generator_builder(
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
          url: p.url.expect("should have url"),
        }) as Box<dyn ParserAndGenerator>
      }),
    );
    ctx.register_parser_and_generator_builder(
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
          url: p.url.expect("should have url"),
        }) as Box<dyn ParserAndGenerator>
      }),
    );
    ctx.register_parser_and_generator_builder(
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
          url: p.url.expect("should have url"),
        }) as Box<dyn ParserAndGenerator>
      }),
    );

    Ok(())
  }
}
