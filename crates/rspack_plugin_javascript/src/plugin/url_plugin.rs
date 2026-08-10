use concat_string::concat_string;
use rspack_core::{
  ChunkInitFragments, ChunkUkey, CodeGenerationDataFilename, CodeGenerationDataUrl, Compilation,
  CompilationParams, CompilerCompilation, DependencyId, JavascriptParserUrl, Module, ModuleType,
  NormalModuleFactoryParser, ParserAndGenerator, ParserOptions, PathData, Plugin, PublicPath,
  RuntimeCodeTemplate, RuntimeGlobals, SourceType, URLStaticMode, get_css_chunk_filename_template,
  get_js_chunk_filename_template, get_undo_path,
  rspack_sources::{BoxSource, ReplaceSource, SourceExt},
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};

use crate::{
  JavascriptModulesRenderModuleContent, JsPlugin, RenderSource,
  dependency::{
    URL_STATIC_PLACEHOLDER, URL_STATIC_PLACEHOLDER_RE, URLDependency,
    WORKER_STATIC_URL_PLACEHOLDER, WORKER_STATIC_URL_PLACEHOLDER_RE, WorkerDependency,
  },
  parser_and_generator::JavaScriptParserAndGenerator,
};

fn json_string_content(value: &str) -> String {
  let value = rspack_util::json_stringify_str(value);
  value[1..value.len() - 1].to_string()
}

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

async fn get_wasm_output_path(compilation: &Compilation, module: &dyn Module) -> Result<String> {
  let module_id =
    rspack_core::ChunkGraph::get_module_id(&compilation.module_ids_artifact, module.identifier())
      .map(|id| PathData::prepare_id(id.as_str()));
  let mut path_data = PathData::default().module_id_optional(module_id.as_deref());
  if let Some(hash) = &module.build_info().hash {
    let hash = hash.rendered(16);
    path_data = path_data.content_hash(hash).hash(hash);
  }

  compilation
    .get_asset_path(
      &compilation.options.output.webassembly_module_filename,
      path_data,
    )
    .await
}

async fn get_url_dependency_output_path(
  compilation: &Compilation,
  dependency_id: &DependencyId,
  origin_output_path: &str,
  runtime_public_path: &str,
) -> Result<Option<String>> {
  let module_graph = compilation.get_module_graph();
  let dependency = module_graph
    .dependency_by_id(dependency_id)
    .downcast_ref::<URLDependency>()
    .expect("URL static placeholder should belong to URLDependency");
  let is_new_url_relative = matches!(dependency.mode(), Some(JavascriptParserUrl::NewUrlRelative));
  let relative_prefix = is_new_url_relative.then(|| {
    get_undo_path(
      origin_output_path,
      compilation.options.output.path.to_string(),
      true,
    )
  });
  let Some(module_identifier) = module_graph.module_identifier_by_dependency_id(dependency_id)
  else {
    return Ok(None);
  };
  let Some(block_id) = module_graph.get_parent_block(dependency_id) else {
    return Ok(None);
  };
  let Some(entrypoint) = compilation
    .build_chunk_graph_artifact
    .chunk_graph
    .get_block_chunk_group(
      block_id,
      &compilation.build_chunk_graph_artifact.chunk_group_by_ukey,
    )
  else {
    return Ok(None);
  };
  let chunk_ukey = entrypoint.get_entrypoint_chunk();

  // Asset URLs and filenames are runtime-independent. An inner-graph-inactive URL dependency may
  // not have a code generation result for its async entry runtime, while the same module can still
  // have one for another active URL dependency.
  if let Some(codegen_result) = compilation
    .code_generation_results
    .try_get_one(module_identifier)
  {
    if let Some(url) = codegen_result.data.get::<CodeGenerationDataUrl>() {
      return Ok(Some(if is_new_url_relative {
        json_string_content(url.inner())
      } else {
        rspack_util::json_stringify_str(url.inner())
      }));
    }
    if let Some(filename) = codegen_result.data.get::<CodeGenerationDataFilename>() {
      let value = if is_new_url_relative {
        json_string_content(&format!(
          "{}{}",
          relative_prefix.as_deref().unwrap_or_default(),
          filename.filename()
        ))
      } else if filename.public_path() == crate::runtime::AUTO_PUBLIC_PATH_PLACEHOLDER {
        format!(
          "{runtime_public_path} + {}",
          rspack_util::json_stringify_str(filename.filename())
        )
      } else {
        rspack_util::json_stringify_str(&format!(
          "{}{}",
          filename.public_path(),
          filename.filename()
        ))
      };
      return Ok(Some(value));
    }
  }

  let Some(module) = module_graph.module_by_identifier(module_identifier) else {
    return Ok(None);
  };
  if module.identifier().as_str().starts_with("ignored|") {
    return Ok(Some(if is_new_url_relative {
      json_string_content("data:,")
    } else {
      rspack_util::json_stringify_str("data:,")
    }));
  }
  let source_types = module.source_types(module_graph);
  let filename = if source_types.contains(&SourceType::Css) {
    Some(get_css_chunk_output_path(compilation, chunk_ukey).await?)
  } else if source_types.contains(&SourceType::Wasm) {
    Some(get_wasm_output_path(compilation, module.as_ref()).await?)
  } else if source_types.contains(&SourceType::JavaScript) {
    Some(get_chunk_output_path(compilation, chunk_ukey).await?)
  } else {
    None
  };
  if let Some(filename) = filename {
    return Ok(Some(if is_new_url_relative {
      json_string_content(&format!(
        "{}{filename}",
        relative_prefix.as_deref().unwrap_or_default()
      ))
    } else {
      format!(
        "{runtime_public_path} + {}",
        rspack_util::json_stringify_str(&filename)
      )
    }));
  }

  Ok(None)
}

fn is_relative_public_path(public_path: &str) -> bool {
  !public_path.starts_with('/') && url::Url::parse(public_path).is_err()
}

pub async fn replace_static_url_placeholders(
  compilation: &Compilation,
  runtime_template: &RuntimeCodeTemplate,
  output_path: &str,
  source: BoxSource,
) -> Result<BoxSource> {
  let content = source.source().into_string_lossy().into_owned();
  let mut replace_source = ReplaceSource::new(source);
  let module_graph = compilation.get_module_graph();
  let runtime_public_path = runtime_template.render_runtime_globals(&RuntimeGlobals::PUBLIC_PATH);
  let replacements = URL_STATIC_PLACEHOLDER_RE
    .find_iter(&content)
    .map(|cap| (cap.start(), cap.end()));

  for (start, end) in replacements {
    let dep_id = &content[start + URL_STATIC_PLACEHOLDER.len()..end];
    let dep_id: DependencyId = dep_id
      .parse::<u32>()
      .unwrap_or_else(|_| panic!("should be valid dependency id \"{dep_id}\""))
      .into();
    let Some(output_value) =
      get_url_dependency_output_path(compilation, &dep_id, output_path, &runtime_public_path)
        .await?
    else {
      continue;
    };

    replace_source.replace(start as u32, end as u32, output_value, None);
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
  _init_fragments: &mut ChunkInitFragments,
  runtime_template: &RuntimeCodeTemplate,
) -> Result<()> {
  let runtime = compilation
    .build_chunk_graph_artifact
    .chunk_by_ukey
    .expect_get(chunk_ukey)
    .runtime();
  let codegen_result = compilation
    .code_generation_results
    .get(&module.identifier(), Some(runtime));
  if codegen_result.data.contains::<URLStaticMode>() {
    let output_path = get_chunk_output_path(compilation, *chunk_ukey).await?;
    render_source.source = replace_static_url_placeholders(
      compilation,
      runtime_template,
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
      .normal_module_factory_hooks
      .parser
      .tap(normal_module_factory_parser::new(self));
    Ok(())
  }
}
