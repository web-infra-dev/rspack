use std::hash::{Hash, Hasher};

use async_trait::async_trait;
use rayon::prelude::*;
use rspack_core::rspack_sources::{
  BoxSource, ConcatSource, MapOptions, RawSource, Source, SourceExt, SourceMap, SourceMapSource,
  SourceMapSourceOptions,
};
use rspack_core::{
  get_js_chunk_filename_template, runtime_globals, AstOrSource, ChunkKind, FilenameRenderOptions,
  GenerateContext, GenerationResult, Module, ModuleAst, ModuleType, ParseContext, ParseResult,
  ParserAndGenerator, PathData, Plugin, PluginContext, PluginProcessAssetsOutput,
  PluginRenderManifestHookOutput, ProcessAssetsArgs, RenderChunkArgs, RenderManifestEntry,
  SourceType,
};
use rspack_error::{internal_error, IntoTWithDiagnosticArray, Result, TWithDiagnosticArray};
use swc_core::base::{config::JsMinifyOptions, BoolOrDataConfig};
use swc_core::common::util::take::Take;
use swc_core::ecma::ast;
use swc_core::ecma::minifier::option::terser::TerserCompressorOptions;
use tracing::instrument;
use xxhash_rust::xxh3::Xxh3;

use crate::runtime::{generate_chunk_entry_code, render_chunk_modules, render_runtime_modules};
use crate::utils::syntax_by_module_type;
use crate::visitors::{run_after_pass, run_before_pass, DependencyScanner};

#[derive(Debug)]
pub struct JsPlugin {}

impl JsPlugin {
  pub fn new() -> Self {
    Self {}
  }

  pub fn render_require(&self, args: &rspack_core::RenderManifestArgs) -> BoxSource {
    let runtime_requirements = args
      .compilation
      .chunk_graph
      .get_chunk_runtime_requirements(&args.chunk_ukey);

    let strict_module_error_handling = matches!(
      args.compilation.options.output.strict_module_error_handling,
      Some(true)
    );
    let mut sources = ConcatSource::default();

    sources.add(RawSource::from(
      r#"// Check if module is in cache
        var cachedModule = __webpack_module_cache__[moduleId];
        if (cachedModule !== undefined) {
      "#,
    ));

    if strict_module_error_handling {
      sources.add(RawSource::from(
        "if (cachedModule.error !== undefined) throw cachedModule.error;",
      ));
    }

    sources.add(RawSource::from(
      r#"return cachedModule.exports;
      }
      // Create a new module (and put it into the cache)
      var module = (__webpack_module_cache__[moduleId] = {
      "#,
    ));

    if runtime_requirements.contains(runtime_globals::INTERCEPT_MODULE_EXECUTION) {
      sources.add(RawSource::from("id: moduleId,"));
    }

    sources.add(RawSource::from(
      r#"// no module.loaded needed
          exports: {}
        });
        // Execute the module function
      "#,
    ));

    let module_execution = match runtime_requirements
      .contains(runtime_globals::INTERCEPT_MODULE_EXECUTION)
    {
      true => RawSource::from(
        r#"var execOptions = { id: moduleId, module: module, factory: __webpack_modules__[moduleId], require: __webpack_require__ };
            __webpack_require__.i.forEach(function(handler) { handler(execOptions); });
            module = execOptions.module;
            execOptions.factory.call(module.exports, module, module.exports, execOptions.require);
            "#,
      ),
      false => RawSource::from(
        "__webpack_modules__[moduleId](module, module.exports, __webpack_require__);\n",
      ),
    };

    if strict_module_error_handling {
      sources.add(RawSource::from("try {\n"));
      sources.add(module_execution);
      sources.add(RawSource::from(
        r#"} catch (e) {
            module.error = e;
            throw e;
          }
          "#,
      ));
    } else {
      sources.add(module_execution);
    }

    sources.add(RawSource::from(
      "// Return the exports of the module\n return module.exports;\n",
    ));

    sources.boxed()
  }

  pub fn render_bootstrap(&self, args: &rspack_core::RenderManifestArgs) -> BoxSource {
    let runtime_requirements = args
      .compilation
      .chunk_graph
      .get_chunk_runtime_requirements(&args.chunk_ukey);

    let module_factories = runtime_requirements.contains(runtime_globals::MODULE_FACTORIES);

    let mut sources = ConcatSource::default();

    sources.add(RawSource::from(
      "// The module cache\n var __webpack_module_cache__ = {};\n",
    ));
    sources.add(RawSource::from(
      "function __webpack_require__(moduleId) {\n",
    ));
    sources.add(self.render_require(args));
    sources.add(RawSource::from("\n}\n"));

    if module_factories || runtime_requirements.contains(runtime_globals::MODULE_FACTORIES_ADD_ONLY)
    {
      sources.add(RawSource::from(
        "// expose the modules object (__webpack_modules__)\n __webpack_require__.m = __webpack_modules__;\n",
      ));
    }

    if runtime_requirements.contains(runtime_globals::MODULE_CACHE) {
      sources.add(RawSource::from(
        "// expose the module cache\n __webpack_require__.c = __webpack_module_cache__;\n",
      ));
    }

    if runtime_requirements.contains(runtime_globals::INTERCEPT_MODULE_EXECUTION) {
      sources.add(RawSource::from(
        "// expose the module execution interceptor\n __webpack_require__.i = [];\n",
      ));
    }

    sources.boxed()
  }

  pub fn render_main(&self, args: &rspack_core::RenderManifestArgs) -> Result<BoxSource> {
    let compilation = args.compilation;
    let chunk = args.chunk();
    let mut sources = ConcatSource::default();
    sources.add(RawSource::from("var __webpack_modules__ = "));
    sources.add(render_chunk_modules(compilation, &args.chunk_ukey)?);
    sources.add(RawSource::from("\n"));
    sources.add(self.render_bootstrap(args));
    sources.add(render_runtime_modules(compilation, &args.chunk_ukey)?);
    if chunk.has_entry_module(&compilation.chunk_graph) {
      // TODO: how do we handle multiple entry modules?
      sources.add(generate_chunk_entry_code(compilation, &args.chunk_ukey));
    }
    Ok(self.render_iife(sources.boxed(), args))
  }

  pub async fn render_chunk(
    &self,
    args: &rspack_core::RenderManifestArgs<'_>,
  ) -> Result<BoxSource> {
    let source = args
      .compilation
      .plugin_driver
      .clone()
      .read()
      .await
      .render_chunk(RenderChunkArgs {
        compilation: args.compilation,
        chunk_ukey: &args.chunk_ukey,
      })?
      .expect("should has a render_chunk plugin");
    Ok(source)
  }

  pub fn render_iife(
    &self,
    content: BoxSource,
    args: &rspack_core::RenderManifestArgs,
  ) -> BoxSource {
    let mut sources = ConcatSource::default();
    if let Some(library) = &args.compilation.options.output.library && !library.is_empty() {
      sources.add(RawSource::from(format!("var {library};\n")));
    }
    sources.add(RawSource::from("(function() {\n"));
    sources.add(content);
    sources.add(RawSource::from("\n})();\n"));
    sources.boxed()
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

  fn size(&self, module: &dyn Module, _source_type: &SourceType) -> f64 {
    module.original_source().map_or(0, |source| source.size()) as f64
  }

  #[instrument(name = "js:parse", skip_all)]
  fn parse(&mut self, parse_context: ParseContext) -> Result<TWithDiagnosticArray<ParseResult>> {
    let ParseContext {
      source,
      module_identifier,
      module_type,
      resource_data,
      compiler_options,
      ..
    } = parse_context;

    if !module_type.is_js_like() {
      return Err(internal_error!(format!(
        "`module_type` {module_type:?} not supported for `JsParser`"
      )));
    }

    let syntax = syntax_by_module_type(
      &resource_data.resource_path,
      module_type,
      compiler_options.builtins.decorator.is_some(),
    );
    let (mut ast, diagnostics) = match crate::ast::parse(
      source.source().to_string(),
      syntax,
      &resource_data.resource_path.to_string_lossy(),
      module_type,
    ) {
      Ok(ast) => (ast, Vec::new()),
      Err(diagnostics) => (
        rspack_core::ast::javascript::Ast::new(
          ast::Program::Module(ast::Module::dummy()),
          Default::default(),
        ),
        diagnostics.into(),
      ),
    };

    run_before_pass(
      resource_data,
      &mut ast,
      compiler_options,
      syntax,
      parse_context.build_info,
      module_type,
    )?;

    let dep_scanner = ast.visit(|program, context| {
      let mut dep_scanner = DependencyScanner::new(context.unresolved_mark);
      program.visit_with_path(&mut dep_scanner, &mut Default::default());
      dep_scanner
    });

    Ok(
      ParseResult {
        ast_or_source: AstOrSource::Ast(ModuleAst::JavaScript(ast)),
        dependencies: dep_scanner
          .dependencies
          .into_iter()
          .map(|mut d| {
            d.set_parent_module_identifier(Some(module_identifier));
            d
          })
          .collect(),
      }
      .with_diagnostic(diagnostics),
    )
  }

  #[allow(clippy::unwrap_in_result)]
  #[instrument(name = "js:generate", skip_all)]
  fn generate(
    &self,
    ast_or_source: &AstOrSource,
    module: &dyn Module,
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
      run_after_pass(&mut ast, module, generate_context)?;
      let output = crate::ast::stringify(&ast, &generate_context.compilation.options.devtool)?;
      if let Some(map) = output.map {
        Ok(GenerationResult {
          ast_or_source: SourceMapSource::new(SourceMapSourceOptions {
            value: output.code,
            source_map: SourceMap::from_json(&map).map_err(|e| internal_error!(e.to_string()))?,
            name: module.try_as_normal_module()?.request().to_string(),
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
      Err(internal_error!(
        "Unsupported source type {:?} for plugin JavaScript",
        generate_context.requested_source_type,
      ))
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

  async fn content_hash(
    &self,
    _ctx: rspack_core::PluginContext,
    args: &mut rspack_core::ContentHashArgs<'_>,
  ) -> rspack_core::PluginContentHashHookOutput {
    let compilation = &mut args.compilation;
    let chunk = compilation
      .chunk_by_ukey
      .get_mut(&args.chunk_ukey)
      .expect("should have chunk");
    let mut hasher = Xxh3::default();

    let mut ordered_modules = compilation.chunk_graph.get_chunk_modules_by_source_type(
      &args.chunk_ukey,
      SourceType::JavaScript,
      &compilation.module_graph,
    );
    ordered_modules.sort_by_key(|m| &m.module_identifier);
    for mgm in ordered_modules {
      if let Some(hash) = compilation
        .module_graph
        .get_module_hash(&mgm.module_identifier)
      {
        hash.hash(&mut hasher);
      }
    }

    for runtime_module_identifier in compilation
      .chunk_graph
      .get_chunk_runtime_modules_in_order(&args.chunk_ukey)
    {
      if let Some(hash) = compilation
        .runtime_module_hashes
        .get(runtime_module_identifier)
      {
        hash.hash(&mut hasher);
      }
    }

    chunk
      .content_hash
      .insert(SourceType::JavaScript, format!("{:x}", hasher.finish()));
    Ok(())
  }

  async fn render_manifest(
    &self,
    _ctx: PluginContext,
    args: rspack_core::RenderManifestArgs<'_>,
  ) -> PluginRenderManifestHookOutput {
    let compilation = args.compilation;
    let chunk = args.chunk();
    let source = if matches!(chunk.kind, ChunkKind::HotUpdate) {
      self.render_chunk(&args).await?
    } else if chunk.has_runtime(&compilation.chunk_group_by_ukey) {
      self.render_main(&args)?
    } else {
      self.render_chunk(&args).await?
    };
    // let hash = Some(get_hash(compilation).to_string());
    // let hash = None;
    // let chunkhash = Some(get_chunkhash(compilation, &args.chunk_ukey, module_graph).to_string());
    // let chunkhash = None;
    // let contenthash = Some(chunk.hash.clone());
    let filename_template = get_js_chunk_filename_template(
      chunk,
      &compilation.options.output,
      &compilation.chunk_group_by_ukey,
    );
    let hash = Some(chunk.get_render_hash());

    let output_path = filename_template.render(FilenameRenderOptions {
      // See https://github.com/webpack/webpack/blob/4b4ca3bb53f36a5b8fc6bc1bd976ed7af161bd80/lib/TemplatedPathPlugin.js#L214
      name: chunk.name_for_filename_template(),
      extension: Some(".js".to_owned()),
      id: chunk.id.clone(),
      contenthash: Some(
        chunk
          .content_hash
          .get(&SourceType::JavaScript)
          .expect("should have chunk javascript content hash")
          .clone(),
      ),
      chunkhash: hash.clone(),
      hash,
      ..Default::default()
    });

    let path_options = PathData {
      chunk_ukey: args.chunk_ukey,
    };
    Ok(vec![RenderManifestEntry::new(
      source,
      output_path,
      path_options,
    )])
  }

  async fn process_assets_stage_optimize_size(
    &mut self,
    _ctx: PluginContext,
    args: ProcessAssetsArgs<'_>,
  ) -> PluginProcessAssetsOutput {
    let compilation = args.compilation;
    let minify = &compilation.options.builtins.minify;
    if !minify.enable {
      return Ok(());
    }

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

        if let Some(original_source) = original.get_source() {
          let input = original_source.source().to_string();
          let input_source_map = original_source.map(&MapOptions::default());
          let output = crate::ast::minify(&JsMinifyOptions {
            compress: BoolOrDataConfig::from_obj(TerserCompressorOptions {
              passes: minify.passes,
              drop_console: minify.drop_console,
              pure_funcs: minify.pure_funcs.clone(),
              ..Default::default()
            }),
            source_map: BoolOrDataConfig::from_bool(input_source_map.is_some()),
            inline_sources_content: true, // Using true so original_source can be None in SourceMapSource
            emit_source_map_columns: !compilation.options.devtool.cheap(),
            ..Default::default()
          }, input, filename)?;
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
        }
        original.get_info_mut().minimized = true;
        Ok(())
      })?;

    Ok(())
  }
}
