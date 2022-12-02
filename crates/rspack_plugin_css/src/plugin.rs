#![allow(clippy::comparison_chain)]
use std::{borrow::Cow, cmp};

use hashbrown::HashSet;
use itertools::Itertools;
use preset_env_base::query::{Query, Targets};
use rayon::prelude::*;
use swc_css::visit::VisitMutWith;
use swc_css_prefixer::{options::Options, prefixer};

use rspack_core::{
  get_css_chunk_filename_template,
  rspack_sources::{
    BoxSource, CachedSource, ConcatSource, MapOptions, RawSource, Source, SourceExt, SourceMap,
    SourceMapSource, SourceMapSourceOptions,
  },
  Chunk, ChunkGraph, Compilation, FilenameRenderOptions, GenerateContext, GenerationResult, Module,
  ModuleGraph, ModuleType, ParseContext, ParseResult, ParserAndGenerator, PathData, Plugin,
  RenderManifestEntry, SourceType,
};
use rspack_error::{Error, IntoTWithDiagnosticArray, Result, TWithDiagnosticArray};
use tracing::instrument;

use crate::{
  pxtorem::{option::PxToRemOption, px_to_rem::px_to_rem},
  visitors::DependencyScanner,
  SWC_COMPILER,
};

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

  pub(crate) fn get_ordered_chunk_css_modules<'module>(
    chunk: &Chunk,
    chunk_graph: &'module ChunkGraph,
    module_graph: &'module ModuleGraph,
    compilation: &Compilation,
  ) -> Vec<Cow<'module, str>> {
    // Align with https://github.com/webpack/webpack/blob/8241da7f1e75c5581ba535d127fa66aeb9eb2ac8/lib/css/CssModulesPlugin.js#L368
    let mut css_modules = chunk_graph
      .get_chunk_modules_iterable_by_source_type(&chunk.ukey, SourceType::Css, module_graph)
      .collect::<Vec<_>>();
    css_modules.sort_by_key(|module| module.identifier());

    let css_modules: Vec<Cow<'module, str>> =
      Self::get_modules_in_order(chunk, css_modules, compilation);

    css_modules
  }

  pub(crate) fn get_modules_in_order<'module>(
    chunk: &Chunk,
    modules: Vec<&'module dyn Module>,
    compilation: &Compilation,
  ) -> Vec<Cow<'module, str>> {
    // Align with https://github.com/webpack/webpack/blob/8241da7f1e75c5581ba535d127fa66aeb9eb2ac8/lib/css/CssModulesPlugin.js#L269
    if modules.is_empty() {
      return vec![];
    };

    let modules_list = modules.into_iter().map(|m| m.identifier()).collect_vec();

    // Get ordered list of modules per chunk group
    // Lists are in reverse order to allow to use Array.pop()
    let mut modules_by_chunk_group = chunk
      .groups
      .iter()
      .filter_map(|ukey| compilation.chunk_group_by_ukey.get(ukey))
      .map(|chunk_group| {
        let sorted_modules = modules_list
          .clone()
          .into_iter()
          .map(|module_id| {
            let order = chunk_group.module_post_order_index(&module_id);
            (module_id, order)
          })
          .sorted_by(|a, b| {
            if b.1 > a.1 {
              std::cmp::Ordering::Less
            } else if b.1 < a.1 {
              std::cmp::Ordering::Greater
            } else {
              std::cmp::Ordering::Equal
            }
          })
          .map(|item| item.0)
          .collect::<Vec<_>>();

        SortedModules {
          set: sorted_modules.clone().into_iter().collect(),
          list: sorted_modules,
        }
      })
      .collect::<Vec<_>>();

    if modules_by_chunk_group.len() == 1 {
      return modules_by_chunk_group[0].list.clone();
    };

    modules_by_chunk_group.sort_by(compare_module_lists);

    let mut final_modules: Vec<Cow<'module, str>> = vec![];

    loop {
      let mut failed_modules: HashSet<Cow<str>> = HashSet::default();
      let list = modules_by_chunk_group[0].list.clone();
      if list.is_empty() {
        // done, everything empty
        break;
      }
      let mut selected_module = list.last().unwrap().clone();
      let mut has_failed = None;
      'outer: loop {
        for SortedModules { set, list } in &modules_by_chunk_group {
          if list.is_empty() {
            continue;
          }
          let last_module = list.last().unwrap().clone();
          if last_module != selected_module {
            continue;
          }
          if !set.contains(&selected_module) {
            continue;
          }
          failed_modules.insert(selected_module.clone());
          if failed_modules.contains(&last_module) {
            // There is a conflict, try other alternatives
            has_failed = Some(last_module);
            continue;
          }
          selected_module = last_module;
          has_failed = None;
          continue 'outer;
        }
        break;
      }
      if let Some(has_failed) = has_failed.clone() {
        // There is a not resolve-able conflict with the selectedModule
        // TODO: we should emit a warning here
        tracing::warn!("Conflicting order between");
        // 		if (compilation) {
        // 			// TODO print better warning
        // 			compilation.warnings.push(
        // 				new Error(
        // 					`chunk ${
        // 						chunk.name || chunk.id
        // 					}\nConflicting order between ${hasFailed.readableIdentifier(
        // 						compilation.requestShortener
        // 					)} and ${selectedModule.readableIdentifier(
        // 						compilation.requestShortener
        // 					)}`
        // 				)
        // 			);
        // 		}

        selected_module = has_failed;
      }
      // Insert the selected module into the final modules list
      final_modules.push(selected_module.clone());
      // Remove the selected module from all lists
      for SortedModules { set, list } in &mut modules_by_chunk_group {
        let last_module = list.last().unwrap();
        if last_module == &selected_module {
          list.pop();
          set.remove(&selected_module);
        } else if has_failed.is_some() && set.contains(&selected_module) {
          let idx = list.iter().position(|m| m == &selected_module);
          if let Some(idx) = idx {
            list.remove(idx);
          }
        }
      }

      modules_by_chunk_group.sort_by(compare_module_lists);
    }
    final_modules
  }
}

pub(crate) static CSS_MODULE_SOURCE_TYPE_LIST: &[SourceType; 2] =
  &[SourceType::JavaScript, SourceType::Css];

#[derive(Debug)]
pub struct CssParserAndGenerator {
  config: CssConfig,
  meta: Option<String>,
}

impl CssParserAndGenerator {
  pub fn new(config: CssConfig) -> Self {
    Self { config, meta: None }
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

impl ParserAndGenerator for CssParserAndGenerator {
  fn source_types(&self) -> &[SourceType] {
    CSS_MODULE_SOURCE_TYPE_LIST
  }

  fn size(&self, module: &dyn Module, source_type: &SourceType) -> f64 {
    match source_type {
      SourceType::JavaScript => {
        // meta + `module.exports = ...`
        self
          .meta
          .as_ref()
          .map(|item| item.len() as f64 + 17.0)
          .unwrap_or(0.0)
      }
      SourceType::Css => module.original_source().map_or(0, |source| source.size()) as f64,
      _ => unreachable!(),
    }
  }

  #[instrument(name = "css:parse", skip_all)]
  fn parse(&mut self, parse_context: ParseContext) -> Result<TWithDiagnosticArray<ParseResult>> {
    let ParseContext {
      source,
      additional_data,
      ..
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

    self.meta = additional_data.and_then(|data| if data.is_empty() { None } else { Some(data) });

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

  #[allow(clippy::unwrap_in_result)]
  #[instrument(name = "css:generate", skip_all)]
  fn generate(
    &self,
    ast_or_source: &rspack_core::AstOrSource,
    module: &dyn rspack_core::Module,
    generate_context: &mut GenerateContext,
  ) -> Result<rspack_core::GenerationResult> {
    let result = match generate_context.requested_source_type {
      SourceType::Css => {
        let devtool = &generate_context.compilation.options.devtool;
        let (code, source_map) = SWC_COMPILER.codegen(
          ast_or_source
            .as_ast()
            .expect("Expected AST for CSS generator, please file an issue.")
            .as_css()
            .expect("Expected CSS AST for CSS generation, please file an issue."),
          crate::SwcCssSourceMapGenConfig {
            enable: devtool.source_map(),
            inline_sources_content: !devtool.no_sources(),
            emit_columns: !devtool.cheap(),
          },
        )?;
        if let Some(source_map) = source_map {
          let source = SourceMapSource::new(SourceMapSourceOptions {
            value: code,
            name: module.try_as_normal_module()?.request().to_string(),
            source_map: SourceMap::from_slice(&source_map)
              .map_err(|e| rspack_error::Error::InternalError(e.to_string()))?,
            // Safety: original source exists in code generation
            original_source: Some(
              module
                .original_source()
                .expect("Failed to get original source, please file an issue.")
                .source()
                .to_string(),
            ),
            // Safety: original source exists in code generation
            inner_source_map: module
              .original_source()
              .expect("Failed to get original source, please file an issue.")
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
        generate_context.requested_source_type
      ))),
    }?;

    Ok(GenerationResult {
      ast_or_source: result.into(),
    })
  }
}

#[async_trait::async_trait]
impl Plugin for CssPlugin {
  fn name(&self) -> &'static str {
    "css"
  }

  fn apply(
    &mut self,
    ctx: rspack_core::PluginContext<&mut rspack_core::ApplyContext>,
  ) -> Result<()> {
    let config = self.config.clone();
    let builder = move || {
      Box::new(CssParserAndGenerator {
        config: config.clone(),
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

    let ordered_modules = Self::get_ordered_chunk_css_modules(
      chunk,
      &compilation.chunk_graph,
      &compilation.module_graph,
      compilation,
    );

    let sources = ordered_modules
      .par_iter()
      .map(|module_id| {
        let code_gen_result = compilation
          .code_generation_results
          .get(module_id, Some(&chunk.runtime))?;

        code_gen_result
          .get(&SourceType::Css)
          .map(|result| result.ast_or_source.clone().try_into_source())
          .transpose()
      })
      .collect::<Result<Vec<Option<BoxSource>>>>()?
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
    // let hash = None;
    // let chunkhash = None;
    // let contenthash = Some(chunk.hash.clone());
    let hash = Some(chunk.get_render_hash());
    if source.source().is_empty() {
      Ok(Default::default())
    } else {
      let filename_template = get_css_chunk_filename_template(
        chunk,
        &args.compilation.options.output,
        &args.compilation.chunk_group_by_ukey,
      );

      let output_path = filename_template.render(FilenameRenderOptions {
        filename: chunk.name.clone(),
        extension: Some(".css".to_owned()),
        id: Some(chunk.id.to_owned()),
        contenthash: hash.clone(),
        chunkhash: hash.clone(),
        hash,
        ..Default::default()
      });

      let path_data = PathData {
        chunk_ukey: args.chunk_ukey,
      };
      Ok(vec![RenderManifestEntry::new(
        source.boxed(),
        output_path,
        path_data,
      )])
    }
  }

  async fn process_assets(
    &mut self,
    _ctx: rspack_core::PluginContext,
    args: rspack_core::ProcessAssetsArgs<'_>,
  ) -> rspack_core::PluginProcessAssetsOutput {
    let compilation = args.compilation;
    let minify = compilation.options.builtins.minify;
    if !minify {
      return Ok(());
    }

    compilation
      .assets
      .par_iter_mut()
      .filter(|(filename, _)| filename.ends_with(".css"))
      .try_for_each(|(filename, original)| -> Result<()> {
        if original.get_info().minimized {
          return Ok(());
        }

        let input = original.get_source().source().to_string();
        let input_source_map = original.get_source().map(&MapOptions::default());
        let minimized_source = SWC_COMPILER.minify(
          filename,
          input,
          input_source_map,
          crate::SwcCssSourceMapGenConfig {
            enable: compilation.options.devtool.source_map(),
            inline_sources_content: !compilation.options.devtool.no_sources(),
            emit_columns: !compilation.options.devtool.cheap(),
          },
        )?;
        original.set_source(minimized_source);
        original.get_info_mut().minimized = true;
        Ok(())
      })?;

    Ok(())
  }
}

struct SortedModules<'me> {
  pub list: Vec<Cow<'me, str>>,
  pub set: HashSet<Cow<'me, str>>,
}

fn compare_module_lists(a: &SortedModules, b: &SortedModules) -> cmp::Ordering {
  let a = &a.list;
  let b = &b.list;
  if a.is_empty() {
    if b.is_empty() {
      cmp::Ordering::Equal
    } else {
      cmp::Ordering::Greater
    }
  } else if b.is_empty() {
    cmp::Ordering::Less
  } else {
    compare_modules_by_identifier(a.last().unwrap(), b.last().unwrap())
  }
}

fn compare_modules_by_identifier(a_id: &str, b_id: &str) -> cmp::Ordering {
  if a_id < b_id {
    cmp::Ordering::Less
  } else if a_id > b_id {
    cmp::Ordering::Greater
  } else {
    cmp::Ordering::Equal
  }
}
