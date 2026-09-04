#![allow(clippy::too_many_arguments)]

use concat_string::concat_string;
use rspack_core::{
  AsyncDependenciesBlock, AsyncModulesArtifact, ChunkInitFragments, ChunkUkey,
  CodeGenerationDataFilename, Compilation, CompilationFinishModules, CompilationParams,
  CompilerCompilation, DependenciesBlock, DependencyId, DependencyParents, EntryOptions,
  ExportsInfoArtifact, GroupOptions, ImportMetaKnownProperties, JavascriptParserUrl, Module,
  ModuleType, NormalModuleFactoryParser, ParserAndGenerator, ParserOptions, PathData, Plugin,
  PublicPath, RuntimeCodeTemplate, RuntimeGlobals, RuntimeSpec, SideEffectsStateArtifact,
  SourceType, URLStaticMode, get_css_chunk_filename_template, get_js_chunk_filename_template,
  get_undo_path,
  rspack_sources::{BoxSource, ReplaceSource, SourceExt},
};
use rspack_error::Result;
use rspack_hash::{HashDigest, RspackHash, RspackHasher};
use rspack_hook::{plugin, plugin_hook};

use crate::{
  JavascriptModulesRenderModuleContent, JsPlugin, RenderSource,
  dependency::{
    URL_STATIC_PLACEHOLDER, URL_STATIC_PLACEHOLDER_RE, URLDependency,
    WORKER_STATIC_URL_PLACEHOLDER, WORKER_STATIC_URL_PLACEHOLDER_RE, WorkerDependency,
  },
  parser_and_generator::JavaScriptParserAndGenerator,
};

#[plugin]
#[derive(Debug, Default)]
pub struct URLPlugin {}

async fn get_chunk_output_path(compilation: &Compilation, chunk_ukey: ChunkUkey) -> Result<String> {
  let chunk = compilation
    .build_chunk_graph_artifact
    .chunk_by_ukey
    .expect_get(&chunk_ukey);
  let filename_template = get_js_chunk_filename_template(
    chunk,
    &compilation.options.output,
    &compilation.build_chunk_graph_artifact.chunk_group_by_ukey,
  );

  compilation
    .get_path(
      &filename_template,
      PathData::default()
        .chunk(chunk_ukey, compilation)
        .chunk_hash_optional(chunk.rendered_hash(
          &compilation.chunk_hashes_artifact,
          compilation.options.output.hash_digest_length,
        ))
        .chunk_id_optional(chunk.id().map(|id| id.as_str()))
        .chunk_name_optional(chunk.name_for_filename_template())
        .content_hash_optional(chunk.rendered_content_hash_by_source_type(
          &compilation.chunk_hashes_artifact,
          &SourceType::JavaScript,
          compilation.options.output.hash_digest_length,
        ))
        .runtime(chunk.runtime().as_str()),
    )
    .await
}

async fn get_css_chunk_output_path(
  compilation: &Compilation,
  chunk_ukey: ChunkUkey,
) -> Result<String> {
  let chunk = compilation
    .build_chunk_graph_artifact
    .chunk_by_ukey
    .expect_get(&chunk_ukey);
  let filename_template = get_css_chunk_filename_template(
    chunk,
    &compilation.options.output,
    &compilation.build_chunk_graph_artifact.chunk_group_by_ukey,
  );

  compilation
    .get_path(
      filename_template,
      PathData::default()
        .chunk(chunk_ukey, compilation)
        .chunk_hash_optional(chunk.rendered_hash(
          &compilation.chunk_hashes_artifact,
          compilation.options.output.hash_digest_length,
        ))
        .chunk_id_optional(chunk.id().map(|id| id.as_str()))
        .chunk_name_optional(chunk.name_for_filename_template())
        .content_hash_optional(chunk.rendered_content_hash_by_source_type(
          &compilation.chunk_hashes_artifact,
          &SourceType::Css,
          compilation.options.output.hash_digest_length,
        ))
        .runtime(chunk.runtime().as_str()),
    )
    .await
}

fn is_url_entry_module_type(module_type: &ModuleType) -> bool {
  module_type.is_js_like()
    || matches!(
      module_type,
      ModuleType::Css | ModuleType::CssAuto | ModuleType::CssModule | ModuleType::CssGlobal
    )
}

const URL_FINISH_MODULES_STAGE: i32 = -10;

#[plugin_hook(CompilationFinishModules for URLPlugin, stage = URL_FINISH_MODULES_STAGE)]
async fn finish_modules(
  &self,
  compilation: &mut Compilation,
  _async_modules_artifact: &mut AsyncModulesArtifact,
  _exports_info_artifact: &mut ExportsInfoArtifact,
  _side_effects_state_artifact: &mut SideEffectsStateArtifact,
) -> Result<()> {
  let blocks = {
    let module_graph = compilation.get_module_graph();
    module_graph
      .dependencies()
      .filter_map(|(dependency_id, dependency)| {
        let dependency = dependency.downcast_ref::<URLDependency>()?;
        if module_graph.get_parent_block(&dependency_id).is_some() {
          return None;
        }
        let target_module = module_graph.get_module_by_dependency_id(&dependency_id)?;
        if target_module.as_external_module().is_some()
          || target_module.identifier().as_str().starts_with("ignored|")
          || !is_url_entry_module_type(target_module.module_type())
        {
          return None;
        }
        let origin_module = *module_graph.get_parent_module(&dependency_id)?;
        let request = dependency.request();
        let range = dependency.dependency_range();

        let mut hasher = RspackHasher::from(&compilation.options.output);
        origin_module.hash(&mut hasher);
        request.hash(&mut hasher);
        range.hash(&mut hasher);
        let runtime = format!("url-{}", hasher.digest(&HashDigest::Hex).rendered(16));

        let modifier = format!("url-entry-{}-{}", range.start, range.end);
        let mut block = Box::new(AsyncDependenciesBlock::new(
          origin_module,
          None,
          Some(&modifier),
          Vec::new(),
          Some(request),
        ));
        block.add_dependency_id(dependency_id);
        block.set_group_options(GroupOptions::Entrypoint(Box::new(EntryOptions {
          runtime: Some(runtime.into()),
          ..Default::default()
        })));

        Some((dependency_id, block))
      })
      .collect::<Vec<_>>()
  };

  let module_graph = compilation.get_module_graph_mut();
  for (dependency_id, block) in blocks {
    let origin_module = *block.parent();
    let block_id = block.identifier();
    let module = module_graph
      .module_by_identifier_mut(&origin_module)
      .expect("URL dependency should have an origin module");
    module.remove_dependency_id(dependency_id);
    module.add_block_id(block_id);
    module_graph.set_parents(
      dependency_id,
      DependencyParents {
        block: Some(block_id),
        module: origin_module,
        index_in_block: 0,
      },
    );
    module_graph.add_block(block);
  }

  Ok(())
}

fn is_relative_public_path(public_path: &str) -> bool {
  !public_path.starts_with('/') && url::Url::parse(public_path).is_err()
}

pub async fn replace_static_url_placeholders(
  compilation: &Compilation,
  runtime: Option<&RuntimeSpec>,
  output_path: &str,
  source: BoxSource,
) -> Result<BoxSource> {
  let content = source.source().into_string_lossy().into_owned();
  let mut replace_source = ReplaceSource::new(source);
  let module_graph = compilation.get_module_graph();
  let replacements = URL_STATIC_PLACEHOLDER_RE
    .find_iter(&content)
    .map(|cap| (cap.start(), cap.end()));

  for (start, end) in replacements {
    let dep_id = &content[start + URL_STATIC_PLACEHOLDER.len()..end];
    let dep_id: DependencyId = dep_id
      .parse::<u32>()
      .unwrap_or_else(|_| panic!("should be valid dependency id \"{dep_id}\""))
      .into();
    let Some(module_identifier) = module_graph.module_identifier_by_dependency_id(&dep_id) else {
      continue;
    };
    if let Some(block) = module_graph.get_parent_block(&dep_id) {
      let chunk_ukey = compilation
        .build_chunk_graph_artifact
        .chunk_graph
        .get_block_chunk_group(
          block,
          &compilation.build_chunk_graph_artifact.chunk_group_by_ukey,
        )
        .map(|entrypoint| entrypoint.get_entrypoint_chunk())
        .expect("URL entry should have an entrypoint chunk");
      let target_module = module_graph
        .module_by_identifier(module_identifier)
        .expect("URL entry should have a target module");
      let filename = if matches!(
        target_module.module_type(),
        ModuleType::Css | ModuleType::CssAuto | ModuleType::CssModule | ModuleType::CssGlobal
      ) {
        get_css_chunk_output_path(compilation, chunk_ukey).await?
      } else {
        get_chunk_output_path(compilation, chunk_ukey).await?
      };
      replace_source.replace(start as u32, end as u32, filename, None);
      continue;
    }
    // The asset may be extracted into a shared chunk whose runtime is the union
    // of the referencing chunks' runtimes. Fall back to the unique code generation
    // result when the referencing runtime has no exact entry.
    let codegen_result = compilation
      .code_generation_results
      .try_get(module_identifier, runtime)
      .or_else(|_| {
        compilation
          .code_generation_results
          .try_get(module_identifier, None)
      })?;
    let Some(filename) = codegen_result.data().get::<CodeGenerationDataFilename>() else {
      unreachable!()
    };

    replace_source.replace(
      start as u32,
      end as u32,
      filename.filename().to_string(),
      None,
    );
  }

  let worker_replacements = WORKER_STATIC_URL_PLACEHOLDER_RE
    .find_iter(&content)
    .map(|cap| (cap.start(), cap.end()));

  for (start, end) in worker_replacements {
    let dep_id = &content[start + WORKER_STATIC_URL_PLACEHOLDER.len()..end];
    let dep_id: DependencyId = dep_id
      .parse::<u32>()
      .unwrap_or_else(|_| panic!("should be valid dependency id \"{dep_id}\""))
      .into();
    let worker_dep = module_graph
      .dependency_by_id(&dep_id)
      .downcast_ref::<WorkerDependency>()
      .expect("should be WorkerDependency");
    let worker_public_path = worker_dep.public_path().to_string();
    let worker_chunk_ukey = module_graph
      .get_parent_block(&dep_id)
      .and_then(|block| {
        compilation
          .build_chunk_graph_artifact
          .chunk_graph
          .get_block_chunk_group(
            block,
            &compilation.build_chunk_graph_artifact.chunk_group_by_ukey,
          )
      })
      .map(|entrypoint| entrypoint.get_entrypoint_chunk())
      .expect("failed to get worker chunk");
    let filename = get_chunk_output_path(compilation, worker_chunk_ukey).await?;
    let public_path = if !worker_public_path.is_empty() {
      worker_public_path
    } else if let PublicPath::Filename(public_path) = &compilation.options.output.public_path {
      PublicPath::ensure_ends_with_slash(
        PublicPath::render_filename(compilation, public_path).await,
      )
    } else {
      String::new()
    };
    let undo_path = if is_relative_public_path(&public_path) {
      get_undo_path(
        output_path,
        compilation.options.output.path.to_string(),
        true,
      )
    } else {
      String::new()
    };

    replace_source.replace(
      start as u32,
      end as u32,
      concat_string!(undo_path, public_path, filename),
      None,
    );
  }

  Ok(replace_source.boxed())
}

#[plugin_hook(CompilerCompilation for URLPlugin)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  let hooks = JsPlugin::get_compilation_hooks_mut(compilation.id());
  hooks
    .write()
    .await
    .render_module_content
    .tap(render_module_content::new(self));
  Ok(())
}
#[plugin_hook(NormalModuleFactoryParser for URLPlugin)]
async fn normal_module_factory_parser(
  &self,
  _module_type: &ModuleType,
  parser: &mut Box<dyn ParserAndGenerator>,
  parser_options: Option<&ParserOptions>,
) -> Result<()> {
  if let Some(parser) = parser.downcast_mut::<JavaScriptParserAndGenerator>() {
    let options = parser_options
      .and_then(|p| p.get_javascript())
      .expect("should at least have a global javascript parser options");

    if !matches!(options.url, Some(JavascriptParserUrl::Disable)) {
      parser.add_parser_plugin(Box::new(crate::parser_plugin::URLPlugin {
        mode: options.url,
        import_meta_url_enabled: options
          .import_meta()
          .is_known_property_enabled(ImportMetaKnownProperties::URL),
      }));
    }
  }

  Ok(())
}

#[plugin_hook(JavascriptModulesRenderModuleContent for URLPlugin,tracing=false)]
async fn render_module_content(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  module: &dyn Module,
  render_source: &mut RenderSource,
  _runtime_requirements: &mut RuntimeGlobals,
  _init_fragments: &mut ChunkInitFragments,
  _runtime_template: &RuntimeCodeTemplate,
) -> Result<()> {
  let runtime = compilation
    .build_chunk_graph_artifact
    .chunk_by_ukey
    .expect_get(chunk_ukey)
    .runtime();
  let codegen_result = compilation
    .code_generation_results
    .get(&module.identifier(), Some(runtime));
  if codegen_result.data().contains::<URLStaticMode>() {
    let output_path = get_chunk_output_path(compilation, *chunk_ukey).await?;
    render_source.source = replace_static_url_placeholders(
      compilation,
      Some(runtime),
      &output_path,
      render_source.source.clone(),
    )
    .await?;
  }
  Ok(())
}

impl Plugin for URLPlugin {
  fn name(&self) -> &'static str {
    "rspack.URLPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compiler_hooks.compilation.tap(compilation::new(self));
    ctx
      .compilation_hooks
      .finish_modules
      .tap(finish_modules::new(self));
    ctx
      .normal_module_factory_hooks
      .parser
      .tap(normal_module_factory_parser::new(self));
    Ok(())
  }
}
