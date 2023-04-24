use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::sync::mpsc;

use async_trait::async_trait;
use linked_hash_set::LinkedHashSet;
use rayon::prelude::*;
use rspack_core::rspack_sources::{
  BoxSource, ConcatSource, MapOptions, RawSource, Source, SourceExt, SourceMap, SourceMapSource,
  SourceMapSourceOptions,
};
use rspack_core::{
  get_js_chunk_filename_template, AstOrSource, ChunkHashArgs, ChunkKind, ChunkUkey, Compilation,
  DependencyType, GenerateContext, GenerationResult, JsChunkHashArgs, Module, ModuleAst,
  ModuleType, ParseContext, ParseResult, ParserAndGenerator, PathData, Plugin,
  PluginChunkHashHookOutput, PluginContext, PluginJsChunkHashHookOutput, PluginProcessAssetsOutput,
  PluginRenderManifestHookOutput, ProcessAssetsArgs, RenderArgs, RenderChunkArgs,
  RenderManifestEntry, RenderStartupArgs, RuntimeGlobals, SourceType,
};
use rspack_error::{
  internal_error, Diagnostic, IntoTWithDiagnosticArray, Result, TWithDiagnosticArray,
};
use rspack_identifier::Identifier;
use swc_core::base::{config::JsMinifyOptions, BoolOrDataConfig};
use swc_core::common::util::take::Take;
use swc_core::ecma::ast;
use swc_core::ecma::minifier::option::terser::TerserCompressorOptions;
use xxhash_rust::xxh3::Xxh3;

use crate::runtime::{generate_chunk_entry_code, render_chunk_modules, render_runtime_modules};
use crate::utils::syntax_by_module_type;
use crate::visitors::{run_after_pass, run_before_pass, scan_dependencies};

#[derive(Debug)]
pub struct JsPlugin {}

impl JsPlugin {
  pub fn new() -> Self {
    Self {}
  }

  pub fn render_require(&self, chunk_ukey: &ChunkUkey, compilation: &Compilation) -> BoxSource {
    let runtime_requirements = compilation
      .chunk_graph
      .get_chunk_runtime_requirements(chunk_ukey);

    let strict_module_error_handling = compilation.options.output.strict_module_error_handling;
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

    if runtime_requirements.contains(RuntimeGlobals::MODULE_ID) {
      sources.add(RawSource::from("id: moduleId,\n"));
    }

    if runtime_requirements.contains(RuntimeGlobals::MODULE_LOADED) {
      sources.add(RawSource::from("loaded: false,\n"));
    }

    sources.add(RawSource::from(
      r#" exports: {} 
      });
      // Execute the module function
      "#,
    ));

    let module_execution = match runtime_requirements
      .contains(RuntimeGlobals::INTERCEPT_MODULE_EXECUTION)
    {
      true => RawSource::from(
        r#"var execOptions = { id: moduleId, module: module, factory: __webpack_modules__[moduleId], require: __webpack_require__ };
            __webpack_require__.i.forEach(function(handler) { handler(execOptions); });
            module = execOptions.module;
            if (!execOptions.factory) {
              console.error("undefined factory", moduleId)
            }
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

    if runtime_requirements.contains(RuntimeGlobals::MODULE_LOADED) {
      sources.add(RawSource::from(
        "// Flag the module as loaded \n module.loaded = true;\n",
      ));
    }

    sources.add(RawSource::from(
      "// Return the exports of the module\n return module.exports;\n",
    ));

    sources.boxed()
  }

  pub fn render_bootstrap(&self, chunk_ukey: &ChunkUkey, compilation: &Compilation) -> BoxSource {
    let runtime_requirements = compilation
      .chunk_graph
      .get_chunk_runtime_requirements(chunk_ukey);

    let module_factories = runtime_requirements.contains(RuntimeGlobals::MODULE_FACTORIES);

    let mut sources = ConcatSource::default();

    sources.add(RawSource::from(
      "// The module cache\n var __webpack_module_cache__ = {};\n",
    ));
    sources.add(RawSource::from(
      "function __webpack_require__(moduleId) {\n",
    ));
    sources.add(self.render_require(chunk_ukey, compilation));
    sources.add(RawSource::from("\n}\n"));

    if module_factories || runtime_requirements.contains(RuntimeGlobals::MODULE_FACTORIES_ADD_ONLY)
    {
      sources.add(RawSource::from(
        "// expose the modules object (__webpack_modules__)\n __webpack_require__.m = __webpack_modules__;\n",
      ));
    }

    if runtime_requirements.contains(RuntimeGlobals::MODULE_CACHE) {
      sources.add(RawSource::from(
        "// expose the module cache\n __webpack_require__.c = __webpack_module_cache__;\n",
      ));
    }

    if runtime_requirements.contains(RuntimeGlobals::INTERCEPT_MODULE_EXECUTION) {
      sources.add(RawSource::from(
        "// expose the module execution interceptor\n __webpack_require__.i = [];\n",
      ));
    }

    sources.boxed()
  }

  pub async fn render_main(&self, args: &rspack_core::RenderManifestArgs<'_>) -> Result<BoxSource> {
    let compilation = args.compilation;
    let chunk = args.chunk();
    let runtime_requirements = compilation
      .chunk_graph
      .get_tree_runtime_requirements(&args.chunk_ukey);
    let mut sources = ConcatSource::default();
    sources.add(RawSource::from("var __webpack_modules__ = "));
    sources.add(render_chunk_modules(compilation, &args.chunk_ukey)?);
    sources.add(RawSource::from("\n"));
    sources.add(self.render_bootstrap(&args.chunk_ukey, args.compilation));
    sources.add(render_runtime_modules(compilation, &args.chunk_ukey)?);
    if chunk.has_entry_module(&compilation.chunk_graph) {
      // TODO: how do we handle multiple entry modules?
      sources.add(generate_chunk_entry_code(compilation, &args.chunk_ukey));
      if runtime_requirements.contains(RuntimeGlobals::RETURN_EXPORTS_FROM_RUNTIME) {
        sources.add(RawSource::from("return __webpack_exports__;\n"));
      }
      let last_entry_module = compilation
        .chunk_graph
        .get_chunk_entry_modules_with_chunk_group_iterable(&chunk.ukey)
        .keys()
        .last()
        .expect("should have last entry module");
      if let Some(source) =
        compilation
          .plugin_driver
          .read()
          .await
          .render_startup(RenderStartupArgs {
            compilation,
            chunk: &chunk.ukey,
            module: *last_entry_module,
          })?
      {
        sources.add(source);
      }
    }
    let final_source = if compilation.options.output.iife {
      self.render_iife(sources.boxed())
    } else {
      sources.boxed()
    };
    if let Some(source) = compilation.plugin_driver.read().await.render(RenderArgs {
      compilation,
      chunk: &args.chunk_ukey,
      source: &final_source,
    })? {
      return Ok(source);
    }
    Ok(final_source)
  }

  #[inline]
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
      })
      .await?
      .expect("should run render_chunk hook");
    Ok(source)
  }

  #[inline]
  pub async fn chunk_hash(
    &self,
    chunk_ukey: &ChunkUkey,
    compilation: &Compilation,
    hasher: &mut Xxh3,
  ) -> PluginJsChunkHashHookOutput {
    compilation
      .plugin_driver
      .clone()
      .read()
      .await
      .js_chunk_hash(JsChunkHashArgs {
        compilation,
        chunk_ukey,
        hasher,
      })
  }

  #[inline]
  pub fn update_hash_with_bootstrap(
    &self,
    chunk_ukey: &ChunkUkey,
    compilation: &Compilation,
    hasher: &mut Xxh3,
  ) {
    // sample hash use content
    self.render_bootstrap(chunk_ukey, compilation).hash(hasher);
  }

  pub fn render_iife(&self, content: BoxSource) -> BoxSource {
    let mut sources = ConcatSource::default();
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

  fn parse(&mut self, parse_context: ParseContext) -> Result<TWithDiagnosticArray<ParseResult>> {
    let ParseContext {
      source,
      module_identifier,
      module_type,
      resource_data,
      compiler_options,
      build_info,
      build_meta,
      ..
    } = parse_context;

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
          None,
        ),
        diagnostics.into(),
      ),
    };

    run_before_pass(
      resource_data,
      &mut ast,
      compiler_options,
      syntax,
      build_info,
      build_meta,
      module_type,
    )?;

    let (dependencies, presentational_dependencies) = ast.visit(|program, context| {
      scan_dependencies(
        program,
        context.unresolved_mark,
        resource_data,
        compiler_options,
        module_type,
      )
    });

    Ok(
      ParseResult {
        ast_or_source: AstOrSource::Ast(ModuleAst::JavaScript(ast)),
        dependencies: dependencies
          .into_iter()
          .map(|mut d| {
            d.set_parent_module_identifier(Some(module_identifier));
            d
          })
          .collect(),
        presentational_dependencies,
      }
      .with_diagnostic(diagnostics),
    )
  }

  #[allow(clippy::unwrap_in_result)]
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
            name: module.try_as_normal_module()?.user_request().to_string(),
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
    args: &ChunkHashArgs<'_>,
  ) -> PluginChunkHashHookOutput {
    let mut hasher = Xxh3::default();
    self
      .chunk_hash(&args.chunk_ukey, args.compilation, &mut hasher)
      .await?;
    if args
      .chunk()
      .has_runtime(&args.compilation.chunk_group_by_ukey)
    {
      self.update_hash_with_bootstrap(&args.chunk_ukey, args.compilation, &mut hasher)
    }
    Ok(Some(hasher.finish()))
  }

  async fn content_hash(
    &self,
    _ctx: rspack_core::PluginContext,
    args: &rspack_core::ContentHashArgs<'_>,
  ) -> rspack_core::PluginContentHashHookOutput {
    let compilation = &args.compilation;
    let chunk = args.chunk();
    let mut hasher = Xxh3::default();

    if chunk.has_runtime(&args.compilation.chunk_group_by_ukey) {
      self.update_hash_with_bootstrap(&args.chunk_ukey, args.compilation, &mut hasher)
    } else {
      chunk.id.hash(&mut hasher);
      chunk.ids.hash(&mut hasher);
    }

    self
      .chunk_hash(&args.chunk_ukey, args.compilation, &mut hasher)
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
      format!("{:x}", hasher.finish()),
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
      self.render_chunk(&args).await?
    } else if chunk.has_runtime(&compilation.chunk_group_by_ukey) {
      self.render_main(&args).await?
    } else {
      self.render_chunk(&args).await?
    };

    let filename_template = get_js_chunk_filename_template(
      chunk,
      &compilation.options.output,
      &compilation.chunk_group_by_ukey,
    );

    let output_path = filename_template.render_with_chunk(chunk, ".js", &SourceType::JavaScript);

    let path_options = PathData {
      chunk_ukey: args.chunk_ukey,
    };
    Ok(vec![RenderManifestEntry::new(
      source,
      output_path,
      path_options,
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

  async fn process_assets_stage_optimize_size(
    &mut self,
    _ctx: PluginContext,
    args: ProcessAssetsArgs<'_>,
  ) -> PluginProcessAssetsOutput {
    let compilation = args.compilation;
    let minify_options = &compilation.options.builtins.minify_options;

    if let Some(minify_options) = minify_options {
      let (tx, rx) = mpsc::channel::<Vec<Diagnostic>>();
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
          // In theory, if a js source is minimized it has high possibility has been tree-shaked.
          if original.get_info().minimized {
            return Ok(());
          }

          if let Some(original_source) = original.get_source() {
            let input = original_source.source().to_string();
            let input_source_map = original_source.map(&MapOptions::default());
            let output = match crate::ast::minify(&JsMinifyOptions {
              compress: BoolOrDataConfig::from_obj(compress.clone()),
              source_map: BoolOrDataConfig::from_bool(input_source_map.is_some()),
              inline_sources_content: true, // Using true so original_source can be None in SourceMapSource
              emit_source_map_columns,
              ..Default::default()
            }, input, filename) {
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
    }

    Ok(())
  }
}

#[derive(Debug)]
pub struct InferAsyncModulesPlugin;

#[async_trait::async_trait]
impl Plugin for InferAsyncModulesPlugin {
  fn name(&self) -> &'static str {
    "InferAsyncModulesPlugin"
  }

  async fn finish_modules(&mut self, compilation: &mut Compilation) -> Result<()> {
    // fix: mut for-in
    let mut queue = LinkedHashSet::new();
    let mut uniques = HashSet::new();

    let mut modules: Vec<Identifier> = compilation
      .module_graph
      .module_graph_modules()
      .values()
      .filter(|m| {
        if let Some(meta) = &m.build_meta {
          meta.is_async
        } else {
          false
        }
      })
      .map(|m| m.module_identifier)
      .collect();

    modules.retain(|m| queue.insert(*m));

    let module_graph = &mut compilation.module_graph;

    while let Some(module) = queue.pop_front() {
      module_graph.set_async(&module);
      if let Some(mgm) = module_graph.module_graph_module_by_identifier(&module) {
        mgm
          .incoming_connections_unordered(module_graph)?
          .filter(|con| {
            if let Some(dep) = module_graph.dependency_by_id(&con.dependency_id) {
              *dep.dependency_type() == DependencyType::EsmImport
            } else {
              false
            }
          })
          .for_each(|con| {
            if let Some(id) = &con.original_module_identifier {
              if uniques.insert(*id) {
                queue.insert(*id);
              }
            }
          });
      }
    }
    Ok(())
  }
}
