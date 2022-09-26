use crate::module::{JsModule, JS_MODULE_SOURCE_TYPE_LIST};
use crate::utils::{
  get_swc_compiler, get_wrap_chunk_after, get_wrap_chunk_before, parse_file, wrap_module_function,
};
use crate::visitors::{ClearMark, DefineScanner, DefineTransform};
use crate::{RSPACK_REGISTER, RSPACK_REQUIRE};
use async_trait::async_trait;
use rayon::prelude::*;
use rspack_core::rspack_sources::{
  BoxSource, CachedSource, ConcatSource, MapOptions, RawSource, Source, SourceExt, SourceMap,
  SourceMapSource, SourceMapSourceOptions,
};
use rspack_core::{
  get_chunkhash, get_contenthash, get_hash, BoxModule, ChunkKind, FilenameRenderOptions,
  ModuleType, ParseModuleArgs, Parser, Plugin, PluginContext, PluginProcessAssetsOutput,
  PluginRenderManifestHookOutput, ProcessAssetsArgs, RenderManifestEntry, SourceType,
};
use swc::config::JsMinifyOptions;
use swc::BoolOrDataConfig;

use rspack_error::{Error, IntoTWithDiagnosticArray, Result, TWithDiagnosticArray};
use swc_common::comments::SingleThreadedComments;
use swc_common::Mark;
use swc_ecma_transforms::helpers::{Helpers, HELPERS};
use swc_ecma_transforms::react::{react, Options as ReactOptions};
use swc_ecma_transforms::{react as swc_react, resolver};
use swc_ecma_visit::{as_folder, FoldWith, VisitWith};

#[derive(Debug)]
pub struct JsPlugin {
  unresolved_mark: Mark,
}

impl JsPlugin {
  pub fn new() -> Self {
    Self {
      unresolved_mark: get_swc_compiler().run(Mark::new),
    }
  }
}

impl Default for JsPlugin {
  fn default() -> Self {
    Self::new()
  }
}

#[async_trait]
impl Plugin for JsPlugin {
  fn name(&self) -> &'static str {
    "javascript"
  }
  fn apply(&mut self, ctx: PluginContext<&mut rspack_core::ApplyContext>) -> Result<()> {
    ctx.context.register_parser(
      ModuleType::Js,
      Box::new(JsParser::new(self.unresolved_mark)),
    );
    ctx.context.register_parser(
      ModuleType::Ts,
      Box::new(JsParser::new(self.unresolved_mark)),
    );
    ctx.context.register_parser(
      ModuleType::Tsx,
      Box::new(JsParser::new(self.unresolved_mark)),
    );
    ctx.context.register_parser(
      ModuleType::Jsx,
      Box::new(JsParser::new(self.unresolved_mark)),
    );

    Ok(())
  }

  fn render_manifest(
    &self,
    _ctx: PluginContext,
    args: rspack_core::RenderManifestArgs,
  ) -> PluginRenderManifestHookOutput {
    let compilation = args.compilation;
    let module_graph = &compilation.module_graph;
    let namespace = &compilation.options.output.unique_name;
    let chunk = args.chunk();
    let mut ordered_modules = compilation.chunk_graph.get_chunk_modules_by_source_type(
      &args.chunk_ukey,
      SourceType::JavaScript,
      module_graph,
    );

    ordered_modules.sort_by_key(|m| &m.uri);

    let has_inline_runtime = !compilation.options.target.platform.is_web()
      && matches!(chunk.kind, ChunkKind::Entry { .. });

    let mut module_code_array = ordered_modules
      .par_iter()
      .map(|module| {
        module
          .module
          .render(SourceType::JavaScript, module, compilation)
          .map(|source| source.map(|source| wrap_module_function(source, &module.id)))
      })
      .collect::<Result<Vec<Option<BoxSource>>>>()?;

    if !has_inline_runtime {
      // insert chunk wrapper
      module_code_array.insert(
        0,
        Some(get_wrap_chunk_before(
          namespace,
          RSPACK_REGISTER,
          &args.chunk().id.to_owned(),
          &compilation.options.target.platform,
        )),
      );
      module_code_array.push(Some(get_wrap_chunk_after(
        &compilation.options.target.platform,
      )));
    }

    let sources = module_code_array
      .into_par_iter()
      .flatten()
      .chain([{
        if chunk.kind.is_entry() && !has_inline_runtime {
          // TODO: how do we handle multiple entry modules?
          let entry_module_uri = args
            .compilation
            .chunk_graph
            .get_chunk_entry_modules(&args.chunk_ukey)
            .into_iter()
            .next()
            .unwrap_or_else(|| panic!("entry module not found"));
          let entry_module_id = &args
            .compilation
            .module_graph
            .module_by_uri(entry_module_uri)
            .unwrap_or_else(|| panic!("entry module not found"))
            .id;

          compilation
            .runtime
            .generate_rspack_execute(namespace, RSPACK_REQUIRE, entry_module_id)
        } else {
          RawSource::from(String::new()).boxed()
        }
      }])
      .fold(ConcatSource::default, |mut output, cur| {
        output.add(cur);
        output
      })
      .collect::<Vec<ConcatSource>>();
    let source = CachedSource::new(ConcatSource::new(sources));

    let hash = Some(get_hash(compilation).to_string());
    let chunkhash = Some(get_chunkhash(compilation, &args.chunk_ukey, module_graph).to_string());
    let contenthash = Some(get_contenthash(&source).to_string());

    let output_path = match chunk.kind {
      ChunkKind::Entry { .. } => {
        compilation
          .options
          .output
          .filename
          .render(FilenameRenderOptions {
            filename: Some(args.chunk().id.to_owned()),
            extension: Some(".js".to_owned()),
            id: None,
            contenthash,
            chunkhash,
            hash,
          })
      }
      ChunkKind::Normal => {
        compilation
          .options
          .output
          .chunk_filename
          .render(FilenameRenderOptions {
            filename: None,
            extension: Some(".js".to_owned()),
            id: Some(format!("static/js/{}", args.chunk().id.to_owned())),
            contenthash,
            chunkhash,
            hash,
          })
      }
    };

    Ok(vec![RenderManifestEntry::new(source.boxed(), output_path)])
  }

  async fn process_assets(
    &self,
    _ctx: PluginContext,
    args: ProcessAssetsArgs<'_>,
  ) -> PluginProcessAssetsOutput {
    let compilation = args.compilation;
    let minify = &compilation.options.builtins.minify;
    if !minify {
      return Ok(());
    }

    let swc_compiler = get_swc_compiler();
    let filename_source_pair: Vec<(String, BoxSource)> = compilation
      .assets
      .par_iter()
      .filter(|(filename, _)| {
        filename.ends_with(".js") || filename.ends_with(".cjs") || filename.ends_with(".mjs")
      })
      .map(|(filename, original)| {
        let original_code = original.source().to_string();
        let output =
          swc::try_with_handler(swc_compiler.cm.clone(), Default::default(), |handler| {
            let fm = swc_compiler.cm.new_source_file(
              swc_common::FileName::Custom(filename.to_string()),
              original_code.clone(),
            );
            swc_compiler.minify(
              fm,
              handler,
              &JsMinifyOptions {
                source_map: BoolOrDataConfig::from_bool(true),
                inline_sources_content: true,
                emit_source_map_columns: true,
                ..Default::default()
              },
            )
          })?;
        let source = if let Some(map) = &output.map {
          SourceMapSource::new(SourceMapSourceOptions {
            value: output.code,
            name: format!("<{filename}>"), // match with swc FileName::Custom...
            source_map: SourceMap::from_json(map)
              .map_err(|e| rspack_error::Error::InternalError(e.to_string()))?,
            original_source: Some(original_code),
            inner_source_map: original.map(&MapOptions::default()),
            remove_original_source: true,
          })
          .boxed()
        } else {
          RawSource::from(output.code).boxed()
        };
        Ok((filename.to_string(), source))
      })
      .collect::<Result<Vec<_>>>()?;

    for (filename, source) in filename_source_pair {
      compilation.emit_asset(filename, source)
    }
    Ok(())
  }
}

#[derive(Debug)]
struct JsParser {
  unresolved_mark: Mark,
}

impl JsParser {
  fn new(unresolved_mark: Mark) -> Self {
    Self { unresolved_mark }
  }
}

impl Parser for JsParser {
  fn parse(
    &self,
    module_type: ModuleType,
    args: ParseModuleArgs,
  ) -> Result<TWithDiagnosticArray<BoxModule>> {
    if !module_type.is_js_like() {
      return Err(Error::InternalError(format!(
        "`module_type` {:?} not supported for `JsParser`",
        module_type
      )));
    }

    let ast_with_diagnostics =
      parse_file(args.source.source().to_string(), args.uri, &module_type)?;

    let (ast, diagnostics) = ast_with_diagnostics.split_into_parts();

    let processed_ast = get_swc_compiler().run(|| {
      HELPERS.set(&Helpers::new(true), || {
        let defintions = &args.options.define;
        let mut define_scanner = DefineScanner::new(defintions);
        // TODO: find more suitable position.
        ast.visit_with(&mut define_scanner);
        let mut define_transform = DefineTransform::new(defintions, define_scanner);
        let top_level_mark = Mark::new();
        let mut react_folder = react::<SingleThreadedComments>(
          get_swc_compiler().cm.clone(),
          None,
          ReactOptions {
            development: Some(false),
            runtime: Some(swc_react::Runtime::Classic),
            refresh: None,
            ..Default::default()
          },
          Mark::new(),
        );

        // TODO: the order
        let ast = ast.fold_with(&mut define_transform);
        let ast = ast.fold_with(&mut resolver(Mark::new(), top_level_mark, false));
        let ast = ast.fold_with(&mut react_folder);
        ast.fold_with(&mut as_folder(ClearMark))
      })
    });
    let module: BoxModule = Box::new(JsModule {
      ast: processed_ast,
      uri: args.uri.to_string(),
      module_type,
      source_type_list: JS_MODULE_SOURCE_TYPE_LIST,
      unresolved_mark: self.unresolved_mark,
      loaded_source: args.source,
    });
    Ok(module.with_diagnostic(diagnostics))
  }
}
