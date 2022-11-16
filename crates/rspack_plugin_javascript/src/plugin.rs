use crate::runtime::RSPACK_REGISTER;
use anyhow::anyhow;
use async_trait::async_trait;
use dashmap::DashMap;
use hashbrown::hash_map::DefaultHashBuilder;
use rayon::prelude::*;
use rspack_core::rspack_sources::{
  BoxSource, CachedSource, ConcatSource, MapOptions, RawSource, Source, SourceExt, SourceMap,
  SourceMapSource, SourceMapSourceOptions,
};
use rspack_core::{
  get_contenthash, AstOrSource, ChunkKind, ChunkUkey, Compilation, FilenameRenderOptions,
  GenerateContext, GenerationResult, ModuleAst, ModuleType, NormalModule, ParseContext,
  ParseResult, ParserAndGenerator, Plugin, PluginContext, PluginProcessAssetsOutput,
  PluginRenderManifestHookOutput, ProcessAssetsArgs, RenderManifestEntry, SourceType,
  TargetPlatform, RUNTIME_PLACEHOLDER_INSTALLED_MODULES,
};
use rspack_error::{Error, IntoTWithDiagnosticArray, Result, TWithDiagnosticArray};
use swc::{config::JsMinifyOptions, BoolOrDataConfig};
use swc_common::util::take::Take;
use swc_common::GLOBALS;
use tracing::instrument;

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

  pub fn generate_chunk_entry_code(
    &self,
    compilation: &Compilation,
    chunk_ukey: &ChunkUkey,
  ) -> BoxSource {
    let entry_modules_uri = compilation.chunk_graph.get_chunk_entry_modules(chunk_ukey);
    let entry_modules_id = entry_modules_uri
      .into_iter()
      .filter_map(|entry_module_identifier| {
        compilation
          .module_graph
          .module_graph_module_by_identifier(entry_module_identifier)
          .map(|module| &module.id)
      })
      .collect::<Vec<_>>();
    let namespace = &compilation.options.output.unique_name;
    let context_indent = if matches!(
      compilation.options.target.platform,
      TargetPlatform::Web | TargetPlatform::None
    ) {
      String::from("self")
    } else {
      String::from("this")
    };
    let sources = entry_modules_id
      .iter()
      .map(|id| {
        RawSource::from(format!(
          r#"{}["{}"].{}("{}");"#,
          context_indent, namespace, "__rspack_require__", id
        ))
      })
      .collect::<Vec<_>>();
    let concat = ConcatSource::new(sources);
    concat.boxed()
  }

  pub fn render_main(&self, args: &rspack_core::RenderManifestArgs) -> Result<BoxSource> {
    let compilation = args.compilation;
    let chunk = args.chunk();

    let runtime_modules = compilation
      .chunk_graph
      .get_chunk_runtime_modules_in_order(&args.chunk_ukey)
      .iter()
      .filter_map(|identifier| compilation.runtime_modules.get(identifier))
      .fold(ConcatSource::default(), |mut output, cur| {
        output.add(cur.sources.clone());
        output
      });
    let runtime_source = runtime_modules.source().to_string();
    let modules_code_start = runtime_source
      .find(RUNTIME_PLACEHOLDER_INSTALLED_MODULES)
      .ok_or_else(|| anyhow!("runtime placeholder installed modules not found"))?;
    let modules_code_end = modules_code_start + RUNTIME_PLACEHOLDER_INSTALLED_MODULES.len();
    let mut sources = ConcatSource::new([
      // runtime_source is all runtime code, and it's RawSource, so use RawSource at here is fine.
      RawSource::from(&runtime_source[0..modules_code_start]).boxed(),
      self.render_chunk_modules(args)?,
      RawSource::from(&runtime_source[modules_code_end..runtime_source.len()]).boxed(),
    ]);
    if chunk.has_entry_module(&args.compilation.chunk_graph) {
      // TODO: how do we handle multiple entry modules?
      sources.add(self.generate_chunk_entry_code(compilation, &args.chunk_ukey));
    }
    Ok(self.render_iife(sources.boxed()))
  }

  pub fn render_chunk(&self, args: &rspack_core::RenderManifestArgs) -> Result<BoxSource> {
    let platform = &args.compilation.options.target.platform;
    let mut sources = ConcatSource::default();
    sources.add(get_wrap_chunk_before(
      &args.compilation.options.output.unique_name,
      RSPACK_REGISTER,
      &args.chunk().id.to_owned(),
      platform,
    ));
    sources.add(self.render_chunk_modules(args)?);
    sources.add(get_wrap_chunk_after(platform));
    Ok(sources.boxed())
  }

  pub fn render_iife(&self, content: BoxSource) -> BoxSource {
    let mut sources = ConcatSource::default();
    sources.add(RawSource::from("(function() {"));
    sources.add(content);
    sources.add(RawSource::from("})()"));
    sources.boxed()
  }

  pub fn render_chunk_modules(&self, args: &rspack_core::RenderManifestArgs) -> Result<BoxSource> {
    let compilation = args.compilation;
    let module_graph = &compilation.module_graph;
    let mut ordered_modules = compilation.chunk_graph.get_chunk_modules_by_source_type(
      &args.chunk_ukey,
      SourceType::JavaScript,
      module_graph,
    );
    let chunk = args.chunk();

    ordered_modules.sort_by_key(|m| &m.module_identifier);

    let module_code_array = ordered_modules
      .par_iter()
      .map(|mgm| {
        let code_gen_result = compilation
          .code_generation_results
          .get(&mgm.module_identifier, Some(&chunk.runtime))?;

        code_gen_result
          .get(&SourceType::JavaScript)
          .map(|result| {
            let mut module_source = result.ast_or_source.clone().try_into_source()?;

            if args.compilation.options.devtool.eval()
              && args.compilation.options.devtool.source_map()
            {
              module_source =
                wrap_eval_source_map(module_source, &self.eval_source_map_cache, args.compilation)?;
            }

            if mgm.module_type.is_css() && compilation.options.dev_server.hot {
              // inject css hmr runtime
              module_source = ConcatSource::new([
                module_source,
                RawSource::from(
                  r#"
        if (module.hot) {
          var cssReload = __rspack_require__("/css-hmr")(module.id, {"locals":false});
          module.hot.dispose(cssReload);
          module.hot.accept(undefined, cssReload);
        }
                "#,
                )
                .boxed(),
              ])
              .boxed();
              Ok(wrap_module_function(module_source, &mgm.id))
            } else {
              Ok(wrap_module_function(module_source, &mgm.id))
            }
          })
          .transpose()
      })
      .collect::<Result<Vec<Option<BoxSource>>>>()?;

    let module_sources = module_code_array
      .into_par_iter()
      .flatten()
      .fold(ConcatSource::default, |mut output, cur| {
        output.add(cur);
        output
      })
      .collect::<Vec<ConcatSource>>();

    let mut sources = ConcatSource::default();
    sources.add(RawSource::from("{\n"));
    sources.add(CachedSource::new(ConcatSource::new(module_sources)));
    sources.add(RawSource::from("\n}"));

    Ok(CachedSource::new(sources).boxed())
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

    let syntax = syntax_by_module_type(
      &source.source(),
      module_type,
      compiler_options.builtins.decorator.is_some(),
    );
    let (mut ast, diagnostics) = match crate::ast::parse(
      source.source().to_string(),
      syntax,
      &resource_data.resource_path,
      module_type,
    ) {
      Ok(ast) => (ast, Vec::new()),
      Err(diagnostics) => (
        rspack_core::ast::javascript::Ast::new(swc_ecma_ast::Program::Module(
          swc_ecma_ast::Module::dummy(),
        )),
        diagnostics.into(),
      ),
    };

    run_before_pass(resource_data, &mut ast, compiler_options, syntax)?;

    let dep_scanner = ast.visit(|program, _context| {
      let mut dep_scanner = DependencyScanner::default();
      program.visit_with(&mut dep_scanner);
      dep_scanner
    });

    Ok(
      ParseResult {
        ast_or_source: AstOrSource::Ast(ModuleAst::JavaScript(ast)),
        dependencies: dep_scanner.dependencies.into_iter().collect(),
      }
      .with_diagnostic(diagnostics),
    )
  }

  #[allow(clippy::unwrap_in_result)]
  #[instrument(name = "js:generate")]
  fn generate(
    &self,
    ast_or_source: &AstOrSource,
    module: &NormalModule,
    generate_context: &mut GenerateContext,
  ) -> Result<GenerationResult> {
    if matches!(
      generate_context.requested_source_type,
      SourceType::JavaScript
    ) {
      // TODO: this should only return AST for javascript only, It's a fast pass, defer to another pr to solve this.
      // Ok(ast_or_source.to_owned().into())
      let mut ast = ast_or_source
        .to_owned()
        .try_into_ast()?
        .try_into_javascript()?;
      run_after_pass(&mut ast, module, generate_context);
      let output = crate::ast::stringify(&ast, &generate_context.compilation.options.devtool)?;
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
        generate_context.requested_source_type,
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

  fn render_manifest(
    &self,
    _ctx: PluginContext,
    args: rspack_core::RenderManifestArgs,
  ) -> PluginRenderManifestHookOutput {
    let compilation = args.compilation;
    let chunk = args.chunk();
    let filename = args.chunk().id.to_owned();

    let is_hot_update_chunk = matches!(chunk.kind, ChunkKind::HotUpdate);
    let source = if is_hot_update_chunk {
      let mut source = ConcatSource::default();
      source.add(RawSource::Source(format!(
        "self['hotUpdate']('{}', ",
        filename
      )));
      source.add(self.render_chunk_modules(&args)?);
      source.add(RawSource::Source(");".to_string()));
      source.boxed()
    } else if chunk.has_runtime(&compilation.chunk_group_by_ukey) {
      self.render_main(&args)?
    } else {
      self.render_chunk(&args)?
    };

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
          filename: Some(filename),
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
          id: Some(args.chunk().id.to_owned()),
          contenthash,
          chunkhash,
          hash,
        })
    };

    Ok(vec![RenderManifestEntry::new(source, output_path)])
  }

  async fn process_assets(
    &mut self,
    _ctx: PluginContext,
    args: ProcessAssetsArgs<'_>,
  ) -> PluginProcessAssetsOutput {
    let compilation = args.compilation;
    let minify = compilation.options.builtins.minify;
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
        // In theory, if a js source is minimized it has high possibility has been tree-shaked.
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
