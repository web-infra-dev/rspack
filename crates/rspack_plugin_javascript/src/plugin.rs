use async_trait::async_trait;
use dashmap::DashMap;
use hashbrown::hash_map::DefaultHashBuilder;
use rayon::prelude::*;
use swc::{config::JsMinifyOptions, BoolOrDataConfig};
use swc_common::GLOBALS;
use swc_ecma_visit::VisitAllWith;

use rspack_core::rspack_sources::{
  BoxSource, CachedSource, ConcatSource, MapOptions, RawSource, Source, SourceExt, SourceMap,
  SourceMapSource, SourceMapSourceOptions,
};
use rspack_core::{
  get_contenthash, AstOrSource, Compilation, FilenameRenderOptions, GenerationResult,
  JavascriptAstExtend, ModuleAst, ModuleGraphModule, ModuleType, NormalModule, ParseContext,
  ParseResult, ParserAndGenerator, Plugin, PluginContext, PluginProcessAssetsOutput,
  PluginRenderManifestHookOutput, PluginRenderRuntimeHookOutput, ProcessAssetsArgs,
  RenderManifestEntry, RenderRuntimeArgs, SourceType, RUNTIME_PLACEHOLDER_RSPACK_EXECUTE,
};
use rspack_error::{Error, IntoTWithDiagnosticArray, Result, TWithDiagnosticArray};
use tracing::instrument;

use crate::runtime::{generate_commonjs_runtime, RSPACK_REGISTER};
use crate::utils::{
  get_swc_compiler, get_wrap_chunk_after, get_wrap_chunk_before, syntax_by_module_type,
  wrap_eval_source_map, wrap_module_function,
};
use crate::visitors::{run_after_pass, run_before_pass, DependencyScanner};

#[derive(Debug)]
pub struct JsPlugin {
  eval_source_map_cache: DashMap<Box<dyn Source>, Box<dyn Source>, DefaultHashBuilder>,
}

impl JsPlugin {
  pub fn new() -> Self {
    Self {
      eval_source_map_cache: Default::default(),
    }
  }
}

impl Default for JsPlugin {
  fn default() -> Self {
    Self::new()
  }
}

#[derive(Debug)]
pub struct JavaScriptParserAndGenerator {}

impl JavaScriptParserAndGenerator {
  fn new() -> Self {
    Self {}
  }
}

static SOURCE_TYPES: &[SourceType; 1] = &[SourceType::JavaScript];

impl ParserAndGenerator for JavaScriptParserAndGenerator {
  fn source_types(&self) -> &[SourceType] {
    SOURCE_TYPES
  }

  fn size(&self, module: &NormalModule, _source_type: &SourceType) -> f64 {
    module.original_source().map_or(0, |source| source.size()) as f64
  }

  #[instrument(name = "js:parse")]
  fn parse(&mut self, parse_context: ParseContext) -> Result<TWithDiagnosticArray<ParseResult>> {
    let ParseContext {
      source,
      module_type,
      resource_data,
      compiler_options,
      ..
    } = parse_context;

    if !module_type.is_js_like() {
      return Err(Error::InternalError(format!(
        "`module_type` {:?} not supported for `JsParser`",
        module_type
      )));
    }

    let ast_with_diagnostics = crate::ast::parse(
      source.source().to_string(),
      &resource_data.resource_path,
      module_type,
    )?;

    let (ast, diagnostics) = ast_with_diagnostics.split_into_parts();

    let (processed_ast, top_level_mark, unresolved_mark, globals) = run_before_pass(
      resource_data,
      ast,
      compiler_options,
      syntax_by_module_type(source.source().to_string().as_str(), module_type),
    )?;

    let mut dep_scanner = DependencyScanner::default();
    processed_ast.visit_all_with(&mut dep_scanner);

    Ok(
      ParseResult {
        ast_or_source: AstOrSource::Ast(ModuleAst::JavaScript(JavascriptAstExtend {
          ast: processed_ast,
          top_level_mark,
          unresolved_mark,
        })),
        parse_phase_global: Some(globals),
        dependencies: dep_scanner.dependencies.into_iter().collect(),
      }
      .with_diagnostic(diagnostics),
    )
  }

  #[allow(clippy::unwrap_in_result)]
  #[instrument(name = "js:generate")]
  fn generate(
    &self,
    requested_source_type: SourceType,
    ast_or_source: &AstOrSource,
    mgm: &ModuleGraphModule,
    compilation: &Compilation,
  ) -> Result<GenerationResult> {
    let module = compilation
      .module_graph
      .module_by_identifier(&mgm.module_identifier)
      .ok_or_else(|| Error::InternalError("Failed to get module".to_owned()))?;

    if matches!(requested_source_type, SourceType::JavaScript) {
      // TODO: this should only return AST for javascript only, It's a fast pass, defer to another pr to solve this.
      // Ok(ast_or_source.to_owned().into())

      let ast = ast_or_source
        .to_owned()
        .try_into_ast()?
        .try_into_javascript()?;
      let ast = run_after_pass(ast.ast, mgm, compilation)?;
      let output = crate::ast::stringify(&ast, &compilation.options.devtool)?;

      if let Some(map) = output.map {
        Ok(GenerationResult {
          ast_or_source: SourceMapSource::new(SourceMapSourceOptions {
            value: output.code,
            source_map: SourceMap::from_json(&map)
              .map_err(|e| rspack_error::Error::InternalError(e.to_string()))?,
            name: module.request().to_string(),
            original_source: {
              Some(
                // Safety: you can sure that `build` is called before code generation, so that the `original_source` is exist
                module
                  .original_source()
                  .expect("Failed to get original source, please file an issue.")
                  .source()
                  .to_string(),
              )
            },
            inner_source_map: {
              // Safety: you can sure that `build` is called before code generation, so that the `original_source` is exist
              module
                .original_source()
                .expect("Failed to get original source, please file an issue.")
                .map(&MapOptions::default())
            },
            remove_original_source: false,
          })
          .boxed()
          .into(),
        })
      } else {
        Ok(GenerationResult {
          ast_or_source: RawSource::from(output.code).boxed().into(),
        })
      }
    } else {
      Err(Error::InternalError(format!(
        "Unsupported source type {:?} for plugin JavaScript",
        requested_source_type,
      )))
    }
  }
}

#[async_trait]
impl Plugin for JsPlugin {
  fn name(&self) -> &'static str {
    "javascript"
  }
  fn apply(&mut self, ctx: PluginContext<&mut rspack_core::ApplyContext>) -> Result<()> {
    let create_parser_and_generator =
      move || Box::new(JavaScriptParserAndGenerator::new()) as Box<dyn ParserAndGenerator>;

    ctx
      .context
      .register_parser_and_generator_builder(ModuleType::Js, Box::new(create_parser_and_generator));
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

    Ok(())
  }

  fn render_runtime(
    &self,
    _ctx: PluginContext,
    args: RenderRuntimeArgs,
  ) -> PluginRenderRuntimeHookOutput {
    let sources = args.sources;
    let mut codes = generate_commonjs_runtime();
    let mut execute_code = None;
    let mut result = Vec::with_capacity(sources.len() + codes.len());
    for item in sources {
      if item.source() == RUNTIME_PLACEHOLDER_RSPACK_EXECUTE {
        execute_code = Some(item);
        continue;
      }
      result.push(item);
    }
    result.append(&mut codes);
    if let Some(code) = execute_code {
      result.push(code);
    }
    Ok(result)
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

    // FIXME: clone is not good
    ordered_modules.sort_by_key(|m| m.uri.to_owned());

    let has_inline_runtime = !compilation.options.target.platform.is_web()
      && chunk.is_only_initial(&args.compilation.chunk_group_by_ukey);

    let mut module_code_array = ordered_modules
      .par_iter()
      .map(|mgm| {
        let module = compilation
          .module_graph
          .module_by_identifier(&mgm.module_identifier)
          .ok_or_else(|| Error::InternalError("Failed to get module".to_owned()))
          // FIXME: use result
          .expect("Failed to get module");

        module.code_generation(mgm, compilation).and_then(|source| {
          // TODO: this logic is definitely not performant, move to compilation afterwards
          source
            .inner()
            .get(&SourceType::JavaScript)
            .map(|source| {
              let mut module_source = source.ast_or_source.clone().try_into_source().unwrap();
              if args.compilation.options.devtool.eval()
                && args.compilation.options.devtool.source_map()
              {
                module_source = wrap_eval_source_map(
                  module_source,
                  &self.eval_source_map_cache,
                  args.compilation,
                )?;
              }
              Ok(wrap_module_function(module_source, &mgm.id))
            })
            .transpose()
        })
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
        if chunk.has_entry_module(&args.compilation.chunk_graph) && !has_inline_runtime {
          // TODO: how do we handle multiple entry modules?
          args.compilation.generate_chunk_entry_code(&args.chunk_ukey)
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

    // let hash = Some(get_hash(compilation).to_string());
    let hash = None;
    // let chunkhash = Some(get_chunkhash(compilation, &args.chunk_ukey, module_graph).to_string());
    let chunkhash = None;
    let contenthash = Some(get_contenthash(&source).to_string());
    let output_path = if chunk.is_only_initial(&args.compilation.chunk_group_by_ukey) {
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
    } else {
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
    };

    Ok(vec![RenderManifestEntry::new(source.boxed(), output_path)])
  }

  async fn process_assets(
    &mut self,
    _ctx: PluginContext,
    args: ProcessAssetsArgs<'_>,
  ) -> PluginProcessAssetsOutput {
    let compilation = args.compilation;
    let minify = &compilation.options.builtins.minify;
    if !minify {
      return Ok(());
    }

    let swc_compiler = get_swc_compiler();
    compilation
      .assets
      .par_iter_mut()
      .filter(|(filename, _)| {
        filename.ends_with(".js") || filename.ends_with(".cjs") || filename.ends_with(".mjs")
      })
      .try_for_each(|(filename, original)| -> Result<()> {
        if original.get_info().minimized {
          return Ok(());
        }

        let input = original.get_source().source().to_string();
        let input_source_map = original.get_source().map(&MapOptions::default());
        let output = GLOBALS.set(&Default::default(), || {
          swc::try_with_handler(swc_compiler.cm.clone(), Default::default(), |handler| {
            let fm = swc_compiler.cm.new_source_file(
              swc_common::FileName::Custom(filename.to_string()),
              input.clone(),
            );
            swc_compiler.minify(
              fm,
              handler,
              &JsMinifyOptions {
                source_map: BoolOrDataConfig::from_bool(input_source_map.is_some()),
                inline_sources_content: false, // don't need this since we have inner_source_map in SourceMapSource
                emit_source_map_columns: !compilation.options.devtool.cheap(),
                ..Default::default()
              },
            )
          })
        })?;
        let source = if let Some(map) = &output.map {
          SourceMapSource::new(SourceMapSourceOptions {
            value: output.code,
            name: format!("<{filename}>"), // match with swc FileName::Custom...
            source_map: SourceMap::from_json(map)
              .map_err(|e| rspack_error::Error::InternalError(e.to_string()))?,
            original_source: Some(input),
            inner_source_map: input_source_map,
            remove_original_source: true,
          })
          .boxed()
        } else {
          RawSource::from(output.code).boxed()
        };
        original.set_source(source);
        original.get_info_mut().minimized = true;
        Ok(())
      })?;

    Ok(())
  }
}
