// mod js_module;
// pub use js_module::*;

use crate::{
  module::{CssModule, CSS_MODULE_SOURCE_TYPE_LIST},
  pxtorem::{option::PxToRemOption, px_to_rem::px_to_rem},
  visitors::DependencyScanner,
  SWC_COMPILER,
};

// use anyhow::{Context, Result};
use preset_env_base::query::{Query, Targets};
use rayon::prelude::*;
use rspack_core::{
  get_contenthash,
  rspack_sources::{
    CachedSource, ConcatSource, MapOptions, RawSource, Source, SourceExt, SourceMap,
    SourceMapSource, SourceMapSourceOptions,
  },
  AstOrSource, BoxModule, ChunkKind, FilenameRenderOptions, GenerationResult, ModuleAst,
  ModuleType, ParseContext, ParseModuleArgs, ParseResult, Parser, ParserAndGenerator, Plugin,
  RenderManifestEntry, SourceType,
};
use rspack_error::{Error, IntoTWithDiagnosticArray, Result, TWithDiagnosticArray};

use swc_css::visit::VisitMutWith;

use swc_css_prefixer::{options::Options, prefixer};
#[derive(Debug, Default)]
pub struct CssPlugin {
  config: CssConfig,
}

#[derive(Debug, Clone, Default)]
pub struct PostcssConfig {
  pub pxtorem: Option<PxToRemOption>,
}

#[derive(Debug, Default, Clone)]
pub struct CssConfig {
  pub preset_env: Vec<String>,
  pub postcss: PostcssConfig,
}

impl CssPlugin {
  pub fn new(config: CssConfig) -> Self {
    Self { config }
  }
}

#[derive(Debug)]
pub struct CssParserAndGenerator {
  config: CssConfig,
  source_type_list: &'static [SourceType; 2],
  meta: Option<String>,
}

impl CssParserAndGenerator {
  pub fn new(config: CssConfig) -> Self {
    Self {
      config,
      source_type_list: CSS_MODULE_SOURCE_TYPE_LIST,
      meta: None,
    }
  }

  pub fn get_query(&self) -> Option<Query> {
    // TODO: figure out if the prefixer visitMut is stateless
    // I need to clone the preset_env every time, due to I don't know if it is stateless
    // If it is true, I reduce this clone
    if !self.config.preset_env.is_empty() {
      Some(Query::Multiple(self.config.preset_env.clone()))
    } else {
      None
    }
  }
}

static SOURCE_TYPES: &[SourceType; 2] = &[SourceType::JavaScript, SourceType::Css];

impl ParserAndGenerator for CssParserAndGenerator {
  fn source_types(&self) -> &[SourceType] {
    SOURCE_TYPES
  }

  fn parse(&mut self, parse_context: ParseContext) -> Result<TWithDiagnosticArray<ParseResult>> {
    let ParseContext {
      source,
      module_type,
      resource_data,
      compiler_options,
      meta,
    } = parse_context;

    let content = source.source().to_string();
    let TWithDiagnosticArray {
      inner: mut stylesheet,
      diagnostic,
    } = SWC_COMPILER.parse_file(&parse_context.resource_data.resource_path, content)?;

    if let Some(query) = self.get_query() {
      stylesheet.visit_mut_with(&mut prefixer(Options {
        env: Some(Targets::Query(query)),
      }));
    }

    if let Some(config) = self.config.postcss.pxtorem.clone() {
      stylesheet.visit_mut_with(&mut px_to_rem(config));
    }

    self.source_type_list = CSS_MODULE_SOURCE_TYPE_LIST;
    self.meta = meta.and_then(|data| if data.is_empty() { None } else { Some(data) });

    let mut scanner = DependencyScanner::default();
    stylesheet.visit_mut_with(&mut scanner);

    Ok(
      ParseResult {
        dependencies: scanner.dependencies,
        ast_or_source: stylesheet.into(),
      }
      .with_diagnostic(diagnostic),
    )
  }

  fn generate(
    &self,
    requested_source_type: SourceType,
    ast_or_source: &rspack_core::AstOrSource,
    mgm: &rspack_core::ModuleGraphModule,
    compilation: &rspack_core::Compilation,
  ) -> Result<rspack_core::GenerationResult> {
    // Safety: OriginalSource exists in code generation, and CSS AST is also available from parse.
    let result = match requested_source_type {
      SourceType::Css => {
        let (code, source_map) = SWC_COMPILER.codegen(
          ast_or_source.as_ast().unwrap().as_css().unwrap(),
          compilation
            .options
            .devtool
            .then(|| mgm.module.original_source().unwrap()),
        )?;
        if let Some(source_map) = source_map {
          let source = SourceMapSource::new(SourceMapSourceOptions {
            value: code,
            name: mgm.module.request().to_string(),
            source_map: SourceMap::from_slice(&source_map)
              .map_err(|e| rspack_error::Error::InternalError(e.to_string()))?,
            original_source: Some(mgm.module.original_source().unwrap().source().to_string()),
            inner_source_map: mgm
              .module
              .original_source()
              .unwrap()
              .map(&MapOptions::default()),
            remove_original_source: false,
          })
          .boxed();
          Ok(source)
        } else {
          Ok(RawSource::from(code).boxed())
        }
      }
      // This is just a temporary solution for css-modules
      SourceType::JavaScript => Ok(
        RawSource::from(
          self
            .meta
            .clone()
            .map(|item| format!("module.exports = {};", item))
            .unwrap_or_else(|| "".to_string()),
        )
        .boxed(),
      ),
      _ => Err(Error::InternalError(format!(
        "Unsupported source type: {:?}",
        requested_source_type
      ))),
    }?;

    Ok(GenerationResult {
      ast_or_source: result.into(),
    })
  }
}

impl Plugin for CssPlugin {
  fn name(&self) -> &'static str {
    "css"
  }

  fn apply(
    &mut self,
    ctx: rspack_core::PluginContext<&mut rspack_core::ApplyContext>,
  ) -> Result<()> {
    ctx.context.register_parser(
      ModuleType::Css,
      Box::new(CssParser {
        config: self.config.clone(),
      }),
    );
    let config = self.config.clone();
    let builder = move || {
      Box::new(CssParserAndGenerator {
        config: config.clone(),
        source_type_list: CSS_MODULE_SOURCE_TYPE_LIST,
        meta: None,
      }) as Box<dyn ParserAndGenerator>
    };

    ctx
      .context
      .register_parser_and_generator_builder(ModuleType::Css, Box::new(builder));

    Ok(())
  }

  // fn reuse_ast(&self) -> bool {
  //   true
  // }

  // fn transform_include(&self, uri: &str) -> bool {
  //   let extension = Path::new(uri).extension().unwrap().to_string_lossy();
  //   extension == "css"
  // }

  // fn transform(
  //   &self,
  //   _ctx: rspack_core::PluginContext<&mut NormalModuleFactoryContext>,
  //   args: rspack_core::TransformArgs,
  // ) -> rspack_core::PluginTransformOutput {
  //   if let Some(TransformAst::Css(mut ast)) = args.ast {
  //     if let Some(query) = self.get_query() {
  //       ast.visit_mut_with(&mut prefixer(Options {
  //         env: Some(Targets::Query(query)),
  //       }));
  //     }
  //     Ok({
  //       TransformResult {
  //         content: None,
  //         ast: Some(TransformAst::Css(ast)),
  //       }
  //     })
  //   } else {
  //     Ok({
  //       TransformResult {
  //         content: None,
  //         ast: args.ast,
  //       }
  //     })
  //   }
  // }
  // fn transform(
  //   &self,
  //   _ctx: rspack_core::PluginContext<&mut NormalModuleFactoryContext>,
  //   args: rspack_core::TransformArgs,
  // ) -> rspack_core::PluginTransformOutput {
  //   if let Some(TransformAst::Css(mut ast)) = args.ast {
  //     ast.visit_mut_with(&mut prefixer());
  //     Ok({
  //       TransformResult {
  //         content: None,
  //         ast: Some(TransformAst::Css(ast)),
  //       }
  //     })
  //   } else {
  //     Ok({
  //       TransformResult {
  //         content: None,
  //         ast: args.ast,
  //       }
  //     })
  //   }
  // }

  // fn parse(&self, uri: &str, content: &Content) -> rspack_core::PluginParseOutput {
  //   let content = content
  //     .to_owned()
  //     .try_into_string()
  //     .context("Unable to serialize content as string which is required by plugin css")?;
  //   let stylesheet = SWC_COMPILER.parse_file(uri, content)?;
  //   Ok(TransformAst::Css(stylesheet))
  // }

  fn render_manifest(
    &self,
    _ctx: rspack_core::PluginContext,
    args: rspack_core::RenderManifestArgs,
  ) -> rspack_core::PluginRenderManifestHookOutput {
    let compilation = args.compilation;
    // let module_graph = &compilation.module_graph;
    let chunk = args.chunk();

    let ordered_modules = {
      let modules = args
        .compilation
        .chunk_graph
        .get_chunk_modules_by_source_type(
          &chunk.ukey,
          SourceType::Css,
          &args.compilation.module_graph,
        );

      if chunk.groups.len() > 1 {
        panic!("TODO: Supports multiple ChunkGroup");
      }

      let groups = chunk
        .groups
        .iter()
        .filter_map(|ukey| args.compilation.chunk_group_by_ukey.get(ukey))
        .map(|chunk_group| {
          let mut modules = modules.clone();
          modules.sort_by_key(|mgm| chunk_group.module_post_order_index(mgm.uri.as_str()));
          tracing::debug!(
            "modules: {:#?}",
            modules
              .iter()
              .map(|mgm| (
                mgm.uri.clone(),
                chunk_group.module_post_order_index(mgm.uri.as_str())
              ))
              .collect::<Vec<_>>()
          );
          modules
        });

      groups
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("No groups found"))
    };

    let sources = ordered_modules
      .par_iter()
      .map(|module| {
        module
          .module
          .code_generation(module, compilation)
          .map(|source| {
            // TODO: this logic is definitely not performant, move to compilation afterwards
            source
              .inner()
              .get(&SourceType::Css)
              .map(|source| source.ast_or_source.clone().try_into_source().unwrap())
          })
      })
      .collect::<Result<Vec<_>>>()?
      .into_par_iter()
      .fold(ConcatSource::default, |mut output, cur| {
        if let Some(source) = cur {
          output.add(RawSource::from("\n\n"));
          output.add(source);
        }
        output
      })
      .collect::<Vec<ConcatSource>>();
    let source = CachedSource::new(ConcatSource::new(sources));

    // let hash = Some(get_hash(compilation).to_string());
    // let chunkhash = Some(get_chunkhash(compilation, &args.chunk_ukey, module_graph).to_string());
    let hash = None;
    let chunkhash = None;
    let contenthash = Some(get_contenthash(&source).to_string());

    if source.source().is_empty() {
      Ok(Default::default())
    } else {
      let output_path = match chunk.kind {
        ChunkKind::Entry { .. } => {
          compilation
            .options
            .output
            .filename
            .render(FilenameRenderOptions {
              filename: Some(args.chunk().id.to_owned()),
              extension: Some(".css".to_owned()),
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
              extension: Some(".css".to_owned()),
              id: Some(format!("static/css/{}", args.chunk().id.to_owned())),
              contenthash,
              chunkhash,
              hash,
            })
        }
      };

      Ok(vec![RenderManifestEntry::new(source.boxed(), output_path)])
    }
  }
}

#[derive(Debug)]
struct CssParser {
  config: CssConfig,
}

impl CssParser {
  pub fn get_query(&self) -> Option<Query> {
    // TODO: figure out if the prefixer visitMut is stateless
    // I need to clone the preset_env every time, due to I don't know if it is stateless
    // If it is true, I reduce this clone
    if !self.config.preset_env.is_empty() {
      Some(Query::Multiple(self.config.preset_env.clone()))
    } else {
      None
    }
  }
}

impl Parser for CssParser {
  fn parse(
    &self,
    _module_type: ModuleType,
    args: ParseModuleArgs,
  ) -> Result<TWithDiagnosticArray<BoxModule>> {
    let content = args.source.source().to_string();
    let TWithDiagnosticArray {
      inner: mut stylesheet,
      diagnostic,
    } = SWC_COMPILER.parse_file(args.uri, content)?;

    if let Some(query) = self.get_query() {
      stylesheet.visit_mut_with(&mut prefixer(Options {
        env: Some(Targets::Query(query)),
      }));
    }

    if let Some(config) = self.config.postcss.pxtorem.clone() {
      stylesheet.visit_mut_with(&mut px_to_rem(config));
    }

    let module: BoxModule = Box::new(CssModule {
      ast: stylesheet,
      loaded_source: args.source,
      source_type_list: CSS_MODULE_SOURCE_TYPE_LIST,
      meta: args
        .meta
        .and_then(|data| if data.is_empty() { None } else { Some(data) }),
    });

    Ok(module.with_diagnostic(diagnostic))
  }
}
