#![allow(clippy::comparison_chain)]
use std::cmp;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::bail;
use bitflags::bitflags;
use indexmap::IndexMap;
use itertools::Itertools;
use once_cell::sync::Lazy;
use preset_env_base::query::{Query, Targets};
use rayon::prelude::*;
use regex::Regex;
use rspack_core::{
  ast::css::Ast as CssAst,
  get_css_chunk_filename_template,
  rspack_sources::{
    BoxSource, ConcatSource, MapOptions, RawSource, Source, SourceExt, SourceMap, SourceMapSource,
    SourceMapSourceOptions,
  },
  Chunk, ChunkGraph, ChunkKind, Compilation, FilenameRenderOptions, GenerateContext,
  GenerationResult, Module, ModuleGraph, ModuleType, NormalModuleAstOrSource, ParseContext,
  ParseResult, ParserAndGenerator, PathData, Plugin, RenderManifestEntry, SourceType,
};
use rspack_core::{AstOrSource, Filename, ModuleAst, ModuleDependency, ModuleIdentifier};
use rspack_error::{
  internal_error, Diagnostic, IntoTWithDiagnosticArray, Result, TWithDiagnosticArray,
};
use rspack_identifier::IdentifierSet;
use sugar_path::SugarPath;
use swc_core::css::visit::VisitMutWithPath;
use swc_core::{
  css::{
    modules::CssClassName,
    parser::parser::ParserConfig,
    prefixer::{options::Options, prefixer},
    visit::VisitMutWith,
  },
  ecma::atoms::JsWord,
};
use xxhash_rust::xxh3::Xxh3;

use crate::dependency::{
  collect_dependency_code_generation_visitors, CssComposeDependency,
  DependencyCodeGenerationVisitors, DependencyVisitor,
};
use crate::utils::{css_modules_exports_to_string, ModulesTransformConfig};
use crate::{
  pxtorem::{options::PxToRemOptions, px_to_rem::px_to_rem},
  visitors::analyze_dependencies,
  SWC_COMPILER,
};

static ESCAPE_LOCAL_IDENT_REGEX: Lazy<Regex> =
  Lazy::new(|| Regex::new(r#"[<>:"/\\|?*\.]"#).expect("Invalid regex"));

#[derive(Debug)]
pub struct CssPlugin {
  config: CssConfig,
}

#[derive(Debug, Clone, Default)]
pub struct PostcssConfig {
  pub pxtorem: Option<PxToRemOptions>,
}

#[derive(Debug, Clone)]
pub struct ModulesConfig {
  pub locals_convention: LocalsConvention,
  pub local_ident_name: LocalIdentName,
  pub exports_only: bool,
}

#[derive(Debug, Clone)]
pub struct LocalIdentName(Filename);

impl LocalIdentName {
  pub fn render(&self, options: LocalIdentNameRenderOptions) -> String {
    let mut s = self.0.render(options.filename_options);
    if let Some(local) = options.local {
      s = s.replace("[local]", &local);
    }
    s = ESCAPE_LOCAL_IDENT_REGEX.replace_all(&s, "-").into_owned();
    s
  }
}

impl From<String> for LocalIdentName {
  fn from(value: String) -> Self {
    Self(Filename::from(value))
  }
}

pub struct LocalIdentNameRenderOptions {
  pub filename_options: FilenameRenderOptions,
  pub local: Option<String>,
}

bitflags! {
  struct LocalsConventionFlags: u8 {
    const ASIS = 1 << 0;
    const CAMELCASE = 1 << 1;
    const DASHES = 1 << 2;
  }
}

#[derive(Debug, Clone)]
pub struct LocalsConvention(LocalsConventionFlags);

impl LocalsConvention {
  pub fn as_is(&self) -> bool {
    self.0.contains(LocalsConventionFlags::ASIS)
  }

  pub fn camel_case(&self) -> bool {
    self.0.contains(LocalsConventionFlags::CAMELCASE)
  }

  pub fn dashes(&self) -> bool {
    self.0.contains(LocalsConventionFlags::DASHES)
  }
}

impl FromStr for LocalsConvention {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
    Ok(match s {
      "asIs" => Self(LocalsConventionFlags::ASIS),
      "camelCase" => Self(LocalsConventionFlags::ASIS | LocalsConventionFlags::CAMELCASE),
      "camelCaseOnly" => Self(LocalsConventionFlags::CAMELCASE),
      "dashes" => Self(LocalsConventionFlags::ASIS | LocalsConventionFlags::DASHES),
      "dashesOnly" => Self(LocalsConventionFlags::DASHES),
      _ => bail!("css modules exportsLocalsConvention error"),
    })
  }
}

impl Default for LocalsConvention {
  fn default() -> Self {
    Self(LocalsConventionFlags::ASIS)
  }
}

#[derive(Debug, Clone)]
pub struct CssConfig {
  pub targets: Vec<String>,
  pub postcss: PostcssConfig,
  pub modules: ModulesConfig,
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
  ) -> Vec<ModuleIdentifier> {
    // Align with https://github.com/webpack/webpack/blob/8241da7f1e75c5581ba535d127fa66aeb9eb2ac8/lib/css/CssModulesPlugin.js#L368
    let mut css_modules = chunk_graph
      .get_chunk_modules_iterable_by_source_type(&chunk.ukey, SourceType::Css, module_graph)
      .collect::<Vec<_>>();
    css_modules.sort_unstable_by_key(|module| module.identifier());

    let css_modules: Vec<ModuleIdentifier> =
      Self::get_modules_in_order(chunk, css_modules, compilation);

    css_modules
  }

  pub(crate) fn get_modules_in_order(
    chunk: &Chunk,
    modules: Vec<&dyn Module>,
    compilation: &Compilation,
  ) -> Vec<ModuleIdentifier> {
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
          .filter_map(|module_id| {
            let order = chunk_group.module_post_order_index(&module_id);
            order.map(|order| (module_id, order))
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
          .collect_vec();

        SortedModules {
          set: sorted_modules.clone().into_iter().collect(),
          list: sorted_modules,
        }
      })
      .collect::<Vec<_>>();

    if modules_by_chunk_group.len() == 1 {
      return modules_by_chunk_group[0].list.clone();
    };

    modules_by_chunk_group.sort_unstable_by(compare_module_lists);

    let mut final_modules: Vec<ModuleIdentifier> = vec![];

    loop {
      let mut failed_modules: IdentifierSet = Default::default();
      let list = modules_by_chunk_group[0].list.clone();
      if list.is_empty() {
        // done, everything empty
        break;
      }
      let mut selected_module = *list.last().expect("TODO:");
      let mut has_failed = None;
      'outer: loop {
        for SortedModules { set, list } in &modules_by_chunk_group {
          if list.is_empty() {
            continue;
          }
          let last_module = *list.last().expect("TODO:");
          if last_module != selected_module {
            continue;
          }
          if !set.contains(&selected_module) {
            continue;
          }
          failed_modules.insert(selected_module);
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
      if let Some(has_failed) = has_failed {
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
      final_modules.push(selected_module);
      // Remove the selected module from all lists
      for SortedModules { set, list } in &mut modules_by_chunk_group {
        let last_module = list.last();
        if last_module.map_or(false, |last_module| last_module == &selected_module) {
          list.pop();
          set.remove(&selected_module);
        } else if has_failed.is_some() && set.contains(&selected_module) {
          let idx = list.iter().position(|m| m == &selected_module);
          if let Some(idx) = idx {
            list.remove(idx);
          }
        }
      }

      modules_by_chunk_group.sort_unstable_by(compare_module_lists);
    }
    final_modules
  }
}

pub(crate) static CSS_MODULE_SOURCE_TYPE_LIST: &[SourceType; 2] =
  &[SourceType::JavaScript, SourceType::Css];

pub(crate) static CSS_MODULE_EXPORTS_ONLY_SOURCE_TYPE_LIST: &[SourceType; 1] =
  &[SourceType::JavaScript];

#[derive(Debug)]
pub struct CssParserAndGenerator {
  config: CssConfig,
  meta: Option<String>,
  exports: Option<IndexMap<JsWord, Vec<CssClassName>>>,
}

impl CssParserAndGenerator {
  pub fn new(config: CssConfig) -> Self {
    Self {
      config,
      meta: None,
      exports: None,
    }
  }

  pub fn get_query(&self) -> Option<Query> {
    // TODO: figure out if the prefixer visitMut is stateless
    // I need to clone the preset_env every time, due to I don't know if it is stateless
    // If it is true, I reduce this clone
    if !self.config.targets.is_empty() {
      Some(Query::Multiple(self.config.targets.clone()))
    } else {
      None
    }
  }
}

/// Compat for @rspack/postcss-loader css modules
#[derive(serde::Deserialize)]
struct RspackPostcssModules {
  rspack_postcss_modules: String,
}

impl ParserAndGenerator for CssParserAndGenerator {
  fn source_types(&self) -> &[SourceType] {
    if self.config.modules.exports_only {
      CSS_MODULE_EXPORTS_ONLY_SOURCE_TYPE_LIST
    } else {
      CSS_MODULE_SOURCE_TYPE_LIST
    }
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

  fn parse(&mut self, parse_context: ParseContext) -> Result<TWithDiagnosticArray<ParseResult>> {
    let ParseContext {
      source,
      additional_data,
      module_identifier,
      module_type,
      resource_data,
      compiler_options,
      code_generation_dependencies,
      ..
    } = parse_context;
    let cm: Arc<swc_core::common::SourceMap> = Default::default();
    let content = source.source().to_string();
    let css_modules = matches!(module_type, ModuleType::CssModule);
    let TWithDiagnosticArray {
      inner: mut stylesheet,
      mut diagnostic,
    } = SWC_COMPILER.parse_file(
      cm.clone(),
      &parse_context.resource_data.resource_path.to_string_lossy(),
      content,
      ParserConfig {
        css_modules,
        legacy_ie: true,
        ..Default::default()
      },
    )?;

    if let Some(query) = self.get_query() {
      stylesheet.visit_mut_with(&mut prefixer(Options {
        env: Some(Targets::Query(query)),
      }));
    }

    if let Some(config) = self.config.postcss.pxtorem.clone() {
      stylesheet.visit_mut_with(&mut px_to_rem(config));
    }

    let locals = if css_modules {
      let path = Path::new(&resource_data.resource_path).relative(&compiler_options.context);
      let result = swc_core::css::modules::compile(
        &mut stylesheet,
        ModulesTransformConfig {
          name: path.file_stem().map(|n| n.to_string_lossy().to_string()),
          path: path.parent().map(|p| p.to_string_lossy().to_string() + "/"),
          ext: path
            .extension()
            .map(|e| format!("{}{}", ".", e.to_string_lossy())),
          local_name_ident: &self.config.modules.local_ident_name,
        },
      );
      let mut exports: IndexMap<JsWord, _> = result.renamed.into_iter().collect();
      exports.sort_keys();
      Some(exports)
    } else {
      None
    };

    let mut dependencies = analyze_dependencies(
      &mut stylesheet,
      code_generation_dependencies,
      &mut diagnostic,
    );

    let mut dependencies = if let Some(locals) = &locals && !locals.is_empty() {
      let compose_deps = locals.iter().flat_map(|(_, value)| value).filter_map(|name| if let CssClassName::Import { from, .. } = name {
        Some(box CssComposeDependency::new(from.to_string(), None) as Box<dyn ModuleDependency>)
      } else {
        None
      });
      dependencies.extend(compose_deps);
      dependencies.into_iter().unique().collect()
    } else {
      dependencies
    };
    dependencies.iter_mut().for_each(|dep| {
      dep.set_parent_module_identifier(Some(module_identifier));
    });

    self.meta = additional_data.and_then(|data| if data.is_empty() { None } else { Some(data) });
    self.exports = locals;

    if self.exports.is_some() && let Some(meta) = &self.meta && serde_json::from_str::<RspackPostcssModules>(meta).is_ok() {
      diagnostic.push(Diagnostic::warn("CSS Modules".to_string(), format!("file: {} is using `postcss.modules` and `builtins.css.modules` to process css modules at the same time, rspack will use `builtins.css.modules`'s result.", resource_data.resource_path.display()), 0, 0));
    }

    Ok(
      ParseResult {
        dependencies,
        presentational_dependencies: vec![],
        ast_or_source: AstOrSource::Ast(ModuleAst::Css(CssAst::new(stylesheet, cm))),
      }
      .with_diagnostic(diagnostic),
    )
  }

  #[allow(clippy::unwrap_in_result)]
  fn generate(
    &self,
    ast_or_source: &rspack_core::AstOrSource,
    module: &dyn rspack_core::Module,
    generate_context: &mut GenerateContext,
  ) -> Result<rspack_core::GenerationResult> {
    let result = match generate_context.requested_source_type {
      SourceType::Css => {
        let devtool = &generate_context.compilation.options.devtool;
        let mut ast = ast_or_source
          .to_owned()
          .try_into_ast()
          .expect("Expected AST for CSS generator, please file an issue.")
          .try_into_css()
          .expect("Expected CSS AST for CSS generation, please file an issue.");
        let dependency_visitors =
          collect_dependency_code_generation_visitors(module, generate_context)?;
        let cm = ast.get_context().source_map.clone();
        let stylesheet = ast.get_root_mut();
        let DependencyCodeGenerationVisitors {
          visitors,
          root_visitors,
          ..
        } = dependency_visitors;

        {
          if !visitors.is_empty() {
            stylesheet.visit_mut_with_path(
              &mut DependencyVisitor::new(
                visitors
                  .iter()
                  .map(|(ast_path, visitor)| (ast_path, &**visitor))
                  .collect(),
              ),
              &mut Default::default(),
            );
          }

          for (_, root_visitor) in root_visitors {
            stylesheet.visit_mut_with(&mut root_visitor.create());
          }
        }

        let (code, source_map) = SWC_COMPILER.codegen(
          cm,
          stylesheet,
          crate::SwcCssSourceMapGenConfig {
            enable: devtool.source_map(),
            inline_sources_content: !devtool.no_sources(),
            emit_columns: !devtool.cheap(),
          },
        )?;
        if let Some(source_map) = source_map {
          let source = SourceMapSource::new(SourceMapSourceOptions {
            value: code,
            name: module.try_as_normal_module()?.user_request().to_string(),
            source_map: SourceMap::from_slice(&source_map)
              .map_err(|e| internal_error!(e.to_string()))?,
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
      SourceType::JavaScript => {
        let locals = if let Some(exports) = &self.exports {
          css_modules_exports_to_string(
            exports,
            module,
            generate_context.compilation,
            &self.config.modules.locals_convention,
          )?
        } else if let Some(meta) = &self.meta
          && let Ok(meta) = serde_json::from_str::<RspackPostcssModules>(meta)
        {
          format!("module.exports = {};\n", meta.rspack_postcss_modules)
        } else if generate_context.compilation.options.dev_server.hot  {
          "module.hot.accept();".to_string()
        } else {
          "".to_string()
        };
        Ok(RawSource::from(locals).boxed())
      }
      _ => Err(internal_error!(
        "Unsupported source type: {:?}",
        generate_context.requested_source_type
      )),
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
        exports: None,
      }) as Box<dyn ParserAndGenerator>
    };

    ctx
      .context
      .register_parser_and_generator_builder(ModuleType::Css, Box::new(builder.clone()));
    ctx
      .context
      .register_parser_and_generator_builder(ModuleType::CssModule, Box::new(builder));

    Ok(())
  }

  async fn content_hash(
    &self,
    _ctx: rspack_core::PluginContext,
    args: &rspack_core::ContentHashArgs<'_>,
  ) -> rspack_core::PluginContentHashHookOutput {
    let compilation = &args.compilation;
    let chunk = compilation
      .chunk_by_ukey
      .get(&args.chunk_ukey)
      .expect("should have chunk");
    let ordered_modules = Self::get_ordered_chunk_css_modules(
      chunk,
      &compilation.chunk_graph,
      &compilation.module_graph,
      compilation,
    );
    let mut hasher = Xxh3::default();

    ordered_modules
      .iter()
      .map(|mgm| {
        (
          compilation
            .code_generation_results
            .get_hash(mgm, Some(&chunk.runtime)),
          compilation.chunk_graph.get_module_id(*mgm),
        )
      })
      .for_each(|(current, id)| {
        if let Some(current) = current {
          current.hash(&mut hasher);
          id.hash(&mut hasher);
        }
      });

    Ok(Some((SourceType::Css, format!("{:x}", hasher.finish()))))
  }

  async fn render_manifest(
    &self,
    _ctx: rspack_core::PluginContext,
    args: rspack_core::RenderManifestArgs<'_>,
  ) -> rspack_core::PluginRenderManifestHookOutput {
    let compilation = args.compilation;
    let chunk = args.chunk();
    if matches!(chunk.kind, ChunkKind::HotUpdate) {
      return Ok(vec![]);
    }
    let ordered_modules = Self::get_ordered_chunk_css_modules(
      chunk,
      &compilation.chunk_graph,
      &compilation.module_graph,
      compilation,
    );

    // Early bail if any of the normal modules were failed to build.
    if ordered_modules.iter().any(|ident| {
      args
        .compilation
        .module_graph
        .module_by_identifier(ident)
        .and_then(|module| module.as_normal_module())
        .map(|module| {
          matches!(
            module.ast_or_source(),
            NormalModuleAstOrSource::BuiltFailed(..)
          )
        })
        .unwrap_or(false)
    }) {
      return Ok(vec![]);
    }

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
      .enumerate()
      .fold(ConcatSource::default, |mut output, (idx, cur)| {
        if let Some(source) = cur {
          if idx != 0 {
            output.add(RawSource::from("\n\n"));
          }
          output.add(source);
        }
        output
      })
      .collect::<Vec<ConcatSource>>();
    let source = ConcatSource::new(sources);

    if source.source().is_empty() {
      Ok(Default::default())
    } else {
      let filename_template = get_css_chunk_filename_template(
        chunk,
        &args.compilation.options.output,
        &args.compilation.chunk_group_by_ukey,
      );

      let output_path = filename_template.render_with_chunk(chunk, ".css", &SourceType::Css);

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

  async fn process_assets_stage_optimize_size(
    &mut self,
    _ctx: rspack_core::PluginContext,
    args: rspack_core::ProcessAssetsArgs<'_>,
  ) -> rspack_core::PluginProcessAssetsOutput {
    let compilation = args.compilation;
    let minify_options = &compilation.options.builtins.minify_options;
    if minify_options.is_none() {
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

        if let Some(original_source) = original.get_source() {
          let input = original_source.source().to_string();
          let input_source_map = original_source.map(&MapOptions::default());
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
          original.set_source(Some(minimized_source));
        }
        original.get_info_mut().minimized = true;
        Ok(())
      })?;

    Ok(())
  }
}

struct SortedModules {
  pub list: Vec<ModuleIdentifier>,
  pub set: IdentifierSet,
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
    compare_modules_by_identifier(a.last().expect("TODO:"), b.last().expect("TODO:"))
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
