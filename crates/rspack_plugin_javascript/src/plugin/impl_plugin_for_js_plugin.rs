use std::hash::Hash;

use async_trait::async_trait;
use rspack_core::rspack_sources::BoxSource;
use rspack_core::{
  get_js_chunk_filename_template, AdditionalChunkRuntimeRequirementsArgs, ChunkHashArgs, ChunkKind,
  CompilerOptions, ModuleType, ParserAndGenerator, PathData, Plugin,
  PluginAdditionalChunkRuntimeRequirementsOutput, PluginChunkHashHookOutput, PluginContext,
  PluginRenderManifestHookOutput, RenderManifestEntry, RuntimeGlobals, SourceType,
};
use rspack_error::Result;
use rspack_hash::RspackHash;

use crate::parser_and_generator::JavaScriptParserAndGenerator;
use crate::JsPlugin;

#[async_trait]
impl Plugin for JsPlugin {
  fn name(&self) -> &'static str {
    "javascript"
  }
  fn apply(
    &self,
    ctx: PluginContext<&mut rspack_core::ApplyContext>,
    options: &mut CompilerOptions,
  ) -> Result<()> {
    let create_parser_and_generator =
      move || Box::new(JavaScriptParserAndGenerator::new()) as Box<dyn ParserAndGenerator>;

    if options.should_transform_by_default() {
      ctx.context.register_parser_and_generator_builder(
        ModuleType::Ts,
        Box::new(create_parser_and_generator),
      );
      ctx.context.register_parser_and_generator_builder(
        ModuleType::Tsx,
        Box::new(create_parser_and_generator),
      );
      ctx.context.register_parser_and_generator_builder(
        ModuleType::Jsx,
        Box::new(create_parser_and_generator),
      );
      ctx.context.register_parser_and_generator_builder(
        ModuleType::JsxEsm,
        Box::new(create_parser_and_generator),
      );
      ctx.context.register_parser_and_generator_builder(
        ModuleType::JsxDynamic,
        Box::new(create_parser_and_generator),
      );
    }

    ctx
      .context
      .register_parser_and_generator_builder(ModuleType::Js, Box::new(create_parser_and_generator));
    ctx.context.register_parser_and_generator_builder(
      ModuleType::JsEsm,
      Box::new(create_parser_and_generator),
    );
    ctx.context.register_parser_and_generator_builder(
      ModuleType::JsDynamic,
      Box::new(create_parser_and_generator),
    );

    Ok(())
  }

  async fn chunk_hash(
    &self,
    _ctx: PluginContext,
    args: &mut ChunkHashArgs<'_>,
  ) -> PluginChunkHashHookOutput {
    self
      .get_chunk_hash(&args.chunk_ukey, args.compilation, args.hasher)
      .await?;
    if args
      .chunk()
      .has_runtime(&args.compilation.chunk_group_by_ukey)
    {
      self.update_hash_with_bootstrap(&args.chunk_ukey, args.compilation, args.hasher)
    }
    Ok(())
  }

  async fn content_hash(
    &self,
    _ctx: rspack_core::PluginContext,
    args: &rspack_core::ContentHashArgs<'_>,
  ) -> rspack_core::PluginContentHashHookOutput {
    let compilation = &args.compilation;
    let chunk = args.chunk();
    let mut hasher = RspackHash::from(&compilation.options.output);

    if chunk.has_runtime(&args.compilation.chunk_group_by_ukey) {
      self.update_hash_with_bootstrap(&args.chunk_ukey, args.compilation, &mut hasher)
    } else {
      chunk.id.hash(&mut hasher);
      chunk.ids.hash(&mut hasher);
    }

    self
      .get_chunk_hash(&args.chunk_ukey, args.compilation, &mut hasher)
      .await?;

    let mut ordered_modules = compilation.chunk_graph.get_chunk_modules_by_source_type(
      &args.chunk_ukey,
      SourceType::JavaScript,
      &compilation.module_graph,
    );
    // SAFETY: module identifier is unique
    ordered_modules.sort_unstable_by_key(|m| m.module_identifier.as_str());

    ordered_modules
      .iter()
      .map(|mgm| {
        (
          compilation
            .code_generation_results
            .get_hash(&mgm.module_identifier, Some(&chunk.runtime)),
          compilation.chunk_graph.get_module_id(mgm.module_identifier),
        )
      })
      .for_each(|(current, id)| {
        if let Some(current) = current {
          current.hash(&mut hasher);
          id.hash(&mut hasher);
        }
      });

    for runtime_module_identifier in compilation
      .chunk_graph
      .get_chunk_runtime_modules_in_order(&args.chunk_ukey)
    {
      if let Some((hash, _)) = compilation
        .runtime_module_code_generation_results
        .get(runtime_module_identifier)
      {
        hash.hash(&mut hasher);
      }
    }

    Ok(Some((
      SourceType::JavaScript,
      hasher.digest(&compilation.options.output.hash_digest),
    )))
  }

  async fn render_manifest(
    &self,
    _ctx: PluginContext,
    args: rspack_core::RenderManifestArgs<'_>,
  ) -> PluginRenderManifestHookOutput {
    let compilation = args.compilation;
    let chunk = args.chunk();
    let source = if matches!(chunk.kind, ChunkKind::HotUpdate) {
      self.render_chunk_impl(&args).await?
    } else if chunk.has_runtime(&compilation.chunk_group_by_ukey) {
      self.render_main(&args).await?
    } else {
      self.render_chunk_impl(&args).await?
    };

    let filename_template = get_js_chunk_filename_template(
      chunk,
      &compilation.options.output,
      &compilation.chunk_group_by_ukey,
    );
    let (output_path, mut asset_info) = compilation.get_path_with_info(
      filename_template,
      PathData::default()
        .chunk(chunk)
        .content_hash_optional(
          chunk
            .content_hash
            .get(&SourceType::JavaScript)
            .map(|i| i.rendered(compilation.options.output.hash_digest_length)),
        )
        .runtime(&chunk.runtime),
    );
    asset_info.set_javascript_module(compilation.options.output.module);
    Ok(vec![RenderManifestEntry::new(
      source,
      output_path,
      asset_info,
      false,
      false,
    )])
  }

  fn additional_tree_runtime_requirements(
    &self,
    _ctx: PluginContext,
    args: &mut AdditionalChunkRuntimeRequirementsArgs,
  ) -> PluginAdditionalChunkRuntimeRequirementsOutput {
    let compilation = &mut args.compilation;
    let runtime_requirements = &mut args.runtime_requirements;
    if !runtime_requirements.contains(RuntimeGlobals::STARTUP_NO_DEFAULT)
      && compilation
        .chunk_graph
        .has_chunk_entry_dependent_chunks(args.chunk, &compilation.chunk_group_by_ukey)
    {
      runtime_requirements.insert(RuntimeGlobals::ON_CHUNKS_LOADED);
      runtime_requirements.insert(RuntimeGlobals::REQUIRE);
    }
    Ok(())
  }
}

#[derive(Debug, Clone)]
pub struct ExtractedCommentsInfo {
  pub source: BoxSource,
  pub comments_file_name: String,
}
