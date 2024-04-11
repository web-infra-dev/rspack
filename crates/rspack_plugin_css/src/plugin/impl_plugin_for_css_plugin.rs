#![allow(clippy::comparison_chain)]

use std::hash::Hash;

use async_trait::async_trait;
use rayon::prelude::*;
use rspack_core::rspack_sources::ReplaceSource;
use rspack_core::{
  get_css_chunk_filename_template,
  rspack_sources::{ConcatSource, RawSource, Source, SourceExt},
  Chunk, ChunkKind, Module, ModuleType, ParserAndGenerator, PathData, Plugin, RenderManifestEntry,
  SourceType,
};
use rspack_core::{
  ChunkLoading, ChunkLoadingType, ChunkUkey, Compilation, CompilationContentHash,
  CompilationParams, CompilationRenderManifest, CompilationRuntimeRequirementInTree,
  CompilerOptions, DependencyType, LibIdentOptions, PublicPath, RuntimeGlobals,
};
use rspack_error::{Diagnostic, Result};
use rspack_hash::RspackHash;
use rspack_hook::{plugin_hook, AsyncSeries2};
use rspack_plugin_runtime::is_enabled_for_chunk;
use rustc_hash::FxHashMap;

use crate::parser_and_generator::CssParserAndGenerator;
use crate::runtime::CssLoadingRuntimeModule;
use crate::utils::AUTO_PUBLIC_PATH_PLACEHOLDER_REGEX;
use crate::{plugin::CssPluginInner, CssPlugin};

struct CssModuleDebugInfo<'a> {
  pub module: &'a dyn Module,
}

impl CssPlugin {
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
          .get(module_id, Some(&chunk.runtime));

        Ok(
          code_gen_result
            .get(&SourceType::Css)
            .map(|source| (CssModuleDebugInfo { module: *module }, source)),
        )
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
        |mut acc, (debug_info, cur_source)| {
          let (start, end) = Self::render_module_debug_info(compilation, &debug_info);
          acc.add(start);
          acc.add(cur_source.clone());
          acc.add(RawSource::from("\n"));
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

    start.add(RawSource::from(format!(
      "/* #region {:?} */\n",
      debug_module_id,
    )));

    start.add(RawSource::from(format!(
      "/*\n- type: {}\n*/\n",
      module.module_type(),
    )));

    end.add(RawSource::from(format!(
      "/* #endregion {debug_module_id:?} */\n\n"
    )));

    (start, end)
  }
}

#[plugin_hook(AsyncSeries2<Compilation, CompilationParams> for CssPlugin)]
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
  Ok(())
}

#[plugin_hook(CompilationRuntimeRequirementInTree for CssPlugin)]
fn runtime_requirements_in_tree(
  &self,
  compilation: &mut Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_requirements: &RuntimeGlobals,
  runtime_requirements_mut: &mut RuntimeGlobals,
) -> Result<Option<()>> {
  let chunk_loading_value = ChunkLoading::Enable(ChunkLoadingType::Jsonp);
  let is_enabled_for_chunk = is_enabled_for_chunk(chunk_ukey, &chunk_loading_value, compilation);

  if (runtime_requirements.contains(RuntimeGlobals::HAS_CSS_MODULES)
    || runtime_requirements.contains(RuntimeGlobals::ENSURE_CHUNK_HANDLERS)
    || runtime_requirements.contains(RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS))
    && is_enabled_for_chunk
  {
    runtime_requirements_mut.insert(RuntimeGlobals::PUBLIC_PATH);
    runtime_requirements_mut.insert(RuntimeGlobals::GET_CHUNK_CSS_FILENAME);
    runtime_requirements_mut.insert(RuntimeGlobals::HAS_OWN_PROPERTY);
    runtime_requirements_mut.insert(RuntimeGlobals::MODULE_FACTORIES_ADD_ONLY);
    compilation.add_runtime_module(chunk_ukey, Box::<CssLoadingRuntimeModule>::default())?;
  }

  Ok(None)
}

#[plugin_hook(CompilationContentHash for CssPlugin)]
fn content_hash(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  hashes: &mut FxHashMap<SourceType, RspackHash>,
) -> Result<()> {
  let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);
  let module_graph = compilation.get_module_graph();
  let (ordered_modules, _) = Self::get_ordered_chunk_css_modules(
    chunk,
    &compilation.chunk_graph,
    &module_graph,
    compilation,
  );
  let mut hasher = hashes
    .entry(SourceType::Css)
    .or_insert_with(|| RspackHash::from(&compilation.options.output));

  ordered_modules
    .iter()
    .map(|m| {
      (
        compilation
          .code_generation_results
          .get_hash(&m.identifier(), Some(&chunk.runtime)),
        compilation.chunk_graph.get_module_id(m.identifier()),
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
  let chunk = chunk_ukey.as_ref(&compilation.chunk_by_ukey);
  if matches!(chunk.kind, ChunkKind::HotUpdate) {
    return Ok(());
  }
  let module_graph = compilation.get_module_graph();
  let (ordered_css_modules, conflicts) = Self::get_ordered_chunk_css_modules(
    chunk,
    &compilation.chunk_graph,
    &module_graph,
    compilation,
  );

  // Prevent generating css files for chunks which don't contain css modules.
  if ordered_css_modules.is_empty() {
    return Ok(());
  }

  let source = Self::render_chunk_to_source(compilation, chunk, &ordered_css_modules)?;

  let filename_template = get_css_chunk_filename_template(
    chunk,
    &compilation.options.output,
    &compilation.chunk_group_by_ukey,
  );
  let (output_path, asset_info) = compilation.get_path_with_info(
    filename_template,
    PathData::default()
      .chunk(chunk)
      .content_hash_optional(
        chunk
          .content_hash
          .get(&SourceType::Css)
          .map(|i| i.rendered(compilation.options.output.hash_digest_length)),
      )
      .runtime(&chunk.runtime),
  )?;

  let content = source.source();
  let auto_public_path_matches: Vec<_> = AUTO_PUBLIC_PATH_PLACEHOLDER_REGEX
    .find_iter(&content)
    .map(|mat| (mat.start(), mat.end()))
    .collect();
  let source = if !auto_public_path_matches.is_empty() {
    let mut replace = ReplaceSource::new(source);
    for (start, end) in auto_public_path_matches {
      let relative = PublicPath::render_auto_public_path(compilation, &output_path);
      replace.replace(start as u32, end as u32, &relative, None);
    }
    replace.boxed()
  } else {
    source.boxed()
  };
  if let Some(conflicts) = conflicts {
    diagnostics.extend(conflicts.into_iter().map(|conflict| {
      let chunk = conflict.chunk.as_ref(&compilation.chunk_by_ukey);
      let mg = compilation.get_module_graph();

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
          chunk
            .name
            .as_ref()
            .unwrap_or(chunk.id.as_ref().expect("should have chunk id")),
          failed_module.readable_identifier(&compilation.options.context),
          selected_module.readable_identifier(&compilation.options.context)
        ),
      )
    }));
  }
  manifest.push(RenderManifestEntry::new(
    source.boxed(),
    output_path,
    asset_info,
    false,
    false,
  ));
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
          .and_then(|p| p.get_css(&ModuleType::Css))
          .expect("should have CssParserOptions");
        let g = g
          .and_then(|g| g.get_css(&ModuleType::Css))
          .expect("should have CssGeneratorOptions");
        Box::new(CssParserAndGenerator {
          exports: None,
          convention: g
            .exports_convention
            .expect("should have exports_convention"),
          local_ident_name: None,
          exports_only: g.exports_only.expect("should have exports_only"),
          named_exports: p.named_exports.expect("should have named_exports"),
        }) as Box<dyn ParserAndGenerator>
      }),
    );
    ctx.context.register_parser_and_generator_builder(
      ModuleType::CssModule,
      Box::new(|p, g| {
        let p = p
          .and_then(|p| p.get_css_module(&ModuleType::CssModule))
          .expect("should have CssModuleParserOptions");
        let g = g
          .and_then(|g| g.get_css_module(&ModuleType::CssModule))
          .expect("should have CssModuleGeneratorOptions");
        Box::new(CssParserAndGenerator {
          exports: None,
          convention: g
            .exports_convention
            .expect("should have exports_convention"),
          local_ident_name: Some(
            g.local_ident_name
              .clone()
              .expect("should have local_ident_name"),
          ),
          exports_only: g.exports_only.expect("should have exports_only"),
          named_exports: p.named_exports.expect("should have named_exports"),
        }) as Box<dyn ParserAndGenerator>
      }),
    );
    ctx.context.register_parser_and_generator_builder(
      ModuleType::CssAuto,
      Box::new(|p, g| {
        let p = p
          .and_then(|p| p.get_css_auto(&ModuleType::CssAuto))
          .expect("should have CssAutoParserOptions");
        let g = g
          .and_then(|g| g.get_css_auto(&ModuleType::CssAuto))
          .expect("should have CssAutoGeneratorOptions");
        Box::new(CssParserAndGenerator {
          exports: None,
          convention: g
            .exports_convention
            .expect("should have exports_convention"),
          local_ident_name: Some(
            g.local_ident_name
              .clone()
              .expect("should have local_ident_name"),
          ),
          exports_only: g.exports_only.expect("should have exports_only"),
          named_exports: p.named_exports.expect("should have named_exports"),
        }) as Box<dyn ParserAndGenerator>
      }),
    );

    Ok(())
  }
}
