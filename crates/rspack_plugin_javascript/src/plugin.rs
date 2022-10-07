use async_trait::async_trait;
use rayon::prelude::*;

use swc::{config::JsMinifyOptions, BoolOrDataConfig};
use swc_common::comments::SingleThreadedComments;
use swc_common::{FileName, Mark, GLOBALS};
use swc_ecma_transforms::helpers::{inject_helpers, Helpers};
use swc_ecma_transforms::react::{react, Options as ReactOptions};
use swc_ecma_transforms::{pass::noop, react};
use swc_ecma_transforms::{react as swc_react, resolver};
use swc_ecma_visit::{as_folder, FoldWith, VisitAllWith, VisitWith};

use rspack_core::rspack_sources::{
  BoxSource, CachedSource, ConcatSource, MapOptions, RawSource, Source, SourceExt, SourceMap,
  SourceMapSource, SourceMapSourceOptions,
};
use rspack_core::{
  get_contenthash, AstOrSource, ChunkKind, Compilation, FilenameRenderOptions, GenerationResult,
  ModuleAst, ModuleGraphModule, ModuleType, ParseContext, ParseResult, ParserAndGenerator, Plugin,
  PluginContext, PluginProcessAssetsOutput, PluginRenderManifestHookOutput, ProcessAssetsArgs,
  RenderManifestEntry, SourceType,
};
use rspack_error::{Error, IntoTWithDiagnosticArray, Result, TWithDiagnosticArray};
use tracing::instrument;

use crate::utils::{
  get_swc_compiler, get_wrap_chunk_after, get_wrap_chunk_before, parse_file, syntax_by_module_type,
  wrap_module_function,
};
use crate::visitors::{finalize, ClearMark, DefineScanner, DefineTransform, DependencyScanner};
use crate::{JS_HELPERS, RSPACK_REGISTER, RSPACK_REQUIRE};

#[derive(Debug)]
pub struct JsPlugin {
  unresolved_mark: Mark,
}

impl JsPlugin {
  pub fn new() -> Self {
    Self {
      unresolved_mark: GLOBALS.set(&Default::default(), || get_swc_compiler().run(Mark::new)),
    }
  }
}

impl Default for JsPlugin {
  fn default() -> Self {
    Self::new()
  }
}

#[derive(Debug)]
pub struct JavaScriptParserAndGenerator {
  unresolved_mark: Mark,
}

impl JavaScriptParserAndGenerator {
  fn new(unresolved_mark: Mark) -> Self {
    Self { unresolved_mark }
  }
}

static SOURCE_TYPES: &[SourceType; 1] = &[SourceType::JavaScript];

impl ParserAndGenerator for JavaScriptParserAndGenerator {
  fn source_types(&self) -> &[SourceType] {
    SOURCE_TYPES
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

    let ast_with_diagnostics = parse_file(
      source.source().to_string(),
      &resource_data.resource_path,
      module_type,
    )?;

    let (ast, diagnostics) = ast_with_diagnostics.split_into_parts();

    let processed_ast = GLOBALS.set(&Default::default(), || {
      swc_ecma_transforms::helpers::HELPERS.set(&Helpers::new(true), || {
        let defintions = &compiler_options.define;
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
        let ast = ast.fold_with(&mut inject_helpers());
        ast.fold_with(&mut as_folder(ClearMark))
      })
    });

    let mut dep_scanner = DependencyScanner::default();
    processed_ast.visit_all_with(&mut dep_scanner);

    Ok(
      ParseResult {
        ast_or_source: AstOrSource::Ast(ModuleAst::JavaScript(processed_ast)),
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
    use swc::config::{self as swc_config, SourceMapsConfig};

    if matches!(requested_source_type, SourceType::JavaScript) {
      // TODO: this should only return AST for javascript only, It's a fast pass, defer to another pr to solve this.
      // Ok(ast_or_source.to_owned().into())

      let compiler = get_swc_compiler();
      let output = GLOBALS.set(&Default::default(), || {
        crate::HELPERS.set(&JS_HELPERS, || {
          swc::try_with_handler(compiler.cm.clone(), Default::default(), |handler| {
            let fm = compiler.cm.new_source_file(
              FileName::Custom(mgm.module.request().to_string()),
              mgm.module.request().to_string(),
            );

            compiler.process_js_with_custom_pass(
              fm,
              // TODO: It should have a better way rather than clone.
              Some(
                ast_or_source
                  .to_owned()
                  .try_into_ast()?
                  .try_into_javascript()?,
              ),
              handler,
              &swc_config::Options {
                config: swc_config::Config {
                  jsc: swc_config::JscConfig {
                    target: compilation.options.target.es_version,
                    syntax: Some(syntax_by_module_type(
                      mgm.module.request(),
                      mgm.module.module_type(),
                    )),
                    transform: Some(swc_config::TransformConfig {
                      react: react::Options {
                        runtime: Some(react::Runtime::Automatic),
                        ..Default::default()
                      },
                      ..Default::default()
                    })
                    .into(),
                    ..Default::default()
                  },
                  inline_sources_content: (!compilation.options.devtool.no_sources()).into(),
                  emit_source_map_columns: (!compilation.options.devtool.cheap()).into(),
                  source_maps: Some(SourceMapsConfig::Bool(
                    compilation.options.devtool.source_map(),
                  )),
                  env: if compilation.options.target.platform.is_browsers_list() {
                    Some(swc_ecma_preset_env::Config {
                      mode: if compilation.options.builtins.polyfill {
                        Some(swc_ecma_preset_env::Mode::Usage)
                      } else {
                        Some(swc_ecma_preset_env::Mode::Entry)
                      },
                      targets: Some(swc_ecma_preset_env::Targets::Query(
                        preset_env_base::query::Query::Multiple(
                          compilation.options.builtins.browserslist.clone(),
                        ),
                      )),
                      ..Default::default()
                    })
                  } else {
                    None
                  },
                  ..Default::default()
                },
                // top_level_mark: Some(bundle_ctx.top_level_mark),
                ..Default::default()
              },
              |_, _| noop(),
              |_, _| {
                // noop()
                finalize(mgm, compilation, self.unresolved_mark)
              },
            )
          })
        })
      })?;

      if let Some(map) = output.map {
        Ok(GenerationResult {
          ast_or_source: SourceMapSource::new(SourceMapSourceOptions {
            value: output.code,
            source_map: SourceMap::from_json(&map)
              .map_err(|e| rspack_error::Error::InternalError(e.to_string()))?,
            name: mgm.module.request().to_string(),
            original_source: {
              Some(
                // Safety: you can sure that `build` is called before code generation, so that the `original_source` is exist
                mgm
                  .module
                  .original_source()
                  .expect("Failed to get original source, please file an issue.")
                  .source()
                  .to_string(),
              )
            },
            inner_source_map: {
              // Safety: you can sure that `build` is called before code generation, so that the `original_source` is exist
              mgm
                .module
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
    let unresolved_mark = self.unresolved_mark;

    let create_parser_and_generator = move || {
      Box::new(JavaScriptParserAndGenerator::new(unresolved_mark)) as Box<dyn ParserAndGenerator>
    };

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
          .code_generation(module, compilation)
          .map(|source| {
            // TODO: this logic is definitely not performant, move to compilation afterwards
            source.inner().get(&SourceType::JavaScript).map(|source| {
              wrap_module_function(
                source.ast_or_source.clone().try_into_source().unwrap(),
                &module.id,
              )
            })
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

    // let hash = Some(get_hash(compilation).to_string());
    let hash = None;
    // let chunkhash = Some(get_chunkhash(compilation, &args.chunk_ukey, module_graph).to_string());
    let chunkhash = None;
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
    let filename_source_pair: Vec<(String, BoxSource)> = compilation
      .assets
      .par_iter()
      .filter(|(filename, _)| {
        filename.ends_with(".js") || filename.ends_with(".cjs") || filename.ends_with(".mjs")
      })
      .map(|(filename, original)| {
        let input = original.source().to_string();
        let input_source_map = original.map(&MapOptions::default());
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
        Ok((filename.to_string(), source))
      })
      .collect::<Result<Vec<_>>>()?;

    for (filename, source) in filename_source_pair {
      compilation.emit_asset(filename, source)
    }
    Ok(())
  }
}
