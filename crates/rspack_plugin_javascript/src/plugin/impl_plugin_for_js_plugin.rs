use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{mpsc, Mutex};

use async_trait::async_trait;
use rayon::prelude::*;
use rspack_core::rspack_sources::{
  BoxSource, MapOptions, RawSource, Source, SourceExt, SourceMap, SourceMapSource,
  SourceMapSourceOptions,
};
use rspack_core::{
  get_js_chunk_filename_template, AdditionalChunkRuntimeRequirementsArgs, AssetInfo, ChunkHashArgs,
  ChunkKind, CompilationAsset, JsChunkHashArgs, ModuleType, ParserAndGenerator, PathData, Plugin,
  PluginAdditionalChunkRuntimeRequirementsOutput, PluginChunkHashHookOutput, PluginContext,
  PluginJsChunkHashHookOutput, PluginProcessAssetsOutput, PluginRenderManifestHookOutput,
  ProcessAssetsArgs, RenderManifestEntry, RuntimeGlobals, SourceType,
};
use rspack_error::{internal_error, Diagnostic, Result};
use rspack_hash::RspackHash;
use swc_config::config_types::BoolOrDataConfig;
use swc_ecma_minifier::option::terser::TerserCompressorOptions;

use crate::parser_and_generator::JavaScriptParserAndGenerator;
use crate::parser_and_generator::JavaScriptStringReplaceParserAndGenerator;
use crate::{JsMinifyOptions, JsPlugin};

#[async_trait]
impl Plugin for JsPlugin {
  fn name(&self) -> &'static str {
    "javascript"
  }
  fn apply(&self, ctx: PluginContext<&mut rspack_core::ApplyContext>) -> Result<()> {
    let create_parser_and_generator =
      move || Box::new(JavaScriptParserAndGenerator::new()) as Box<dyn ParserAndGenerator>;

    let create_parser_and_generator = move || {
      Box::new(JavaScriptStringReplaceParserAndGenerator::new()) as Box<dyn ParserAndGenerator>
    };

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
    ctx
      .context
      .register_parser_and_generator_builder(ModuleType::Ts, Box::new(create_parser_and_generator));
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
    let (output_path, asset_info) = compilation.get_path_with_info(
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
    Ok(vec![RenderManifestEntry::new(
      source,
      output_path,
      asset_info,
    )])
  }

  fn js_chunk_hash(
    &self,
    _ctx: PluginContext,
    args: &mut JsChunkHashArgs,
  ) -> PluginJsChunkHashHookOutput {
    self.name().hash(&mut args.hasher);
    args
      .compilation
      .options
      .builtins
      .minify_options
      .hash(&mut args.hasher);
    Ok(())
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

  async fn process_assets_stage_optimize_size(
    &self,
    _ctx: PluginContext,
    args: ProcessAssetsArgs<'_>,
  ) -> PluginProcessAssetsOutput {
    let compilation = args.compilation;
    let minify_options = &compilation.options.builtins.minify_options;

    if let Some(minify_options) = minify_options {
      let (tx, rx) = mpsc::channel::<Vec<Diagnostic>>();
      // collect all extracted comments info
      let all_extracted_comments = Mutex::new(HashMap::new());
      let extract_comments_option = &minify_options.extract_comments.clone();
      let emit_source_map_columns = !compilation.options.devtool.cheap();
      let compress = TerserCompressorOptions {
        passes: minify_options.passes,
        drop_console: minify_options.drop_console,
        pure_funcs: minify_options.pure_funcs.clone(),
        ..Default::default()
      };

      compilation
        .assets_mut()
        .par_iter_mut()
        .filter(|(filename, _)| {
          filename.ends_with(".js") || filename.ends_with(".cjs") || filename.ends_with(".mjs")
        })
        .try_for_each_with(tx, |tx, (filename, original)| -> Result<()> {
          if original.get_info().minimized {
            return Ok(());
          }

          if let Some(original_source) = original.get_source() {
            let input = original_source.source().to_string();
            let input_source_map = original_source.map(&MapOptions::default());
            let js_minify_options = JsMinifyOptions {
              compress: BoolOrDataConfig::from_obj(compress.clone()),
              source_map: BoolOrDataConfig::from_bool(input_source_map.is_some()),
              inline_sources_content: true, // Using true so original_source can be None in SourceMapSource
              emit_source_map_columns,
              ..Default::default()
            };
            let output = match crate::ast::minify(&js_minify_options, input, filename, &all_extracted_comments, extract_comments_option) {
              Ok(r) => r,
              Err(e) => {
                tx.send(e.into()).map_err(|e| internal_error!(e.to_string()))?;
                return Ok(())
              },
            };
            let source = if let Some(map) = &output.map {
              SourceMapSource::new(SourceMapSourceOptions {
                value: output.code,
                name: filename,
                source_map: SourceMap::from_json(map)
                  .map_err(|e| internal_error!(e.to_string()))?,
                original_source: None,
                inner_source_map: input_source_map,
                remove_original_source: true,
              })
              .boxed()
            } else {
              RawSource::from(output.code).boxed()
            };
            original.set_source(Some(source));
            original.get_info_mut().minimized = true;
          }
          Ok(())
        })?;

      compilation.push_batch_diagnostic(rx.into_iter().flatten().collect::<Vec<_>>());

      // write all extracted comments to assets
      all_extracted_comments
        .lock()
        .expect("all_extracted_comments lock failed")
        .clone()
        .into_iter()
        .for_each(|(_, comments)| {
          compilation.emit_asset(
            comments.comments_file_name,
            CompilationAsset {
              source: Some(comments.source),
              info: AssetInfo {
                minimized: true,
                ..Default::default()
              },
            },
          )
        });
    }

    Ok(())
  }
}

#[derive(Debug, Clone)]
pub struct ExtractedCommentsInfo {
  pub source: BoxSource,
  pub comments_file_name: String,
}
