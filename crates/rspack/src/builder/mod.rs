mod devtool;
mod externals;
mod target;

pub use devtool::Devtool;
pub use target::Targets;

macro_rules! d {
  ($o:expr, $v:expr) => {{
    $o.unwrap_or($v)
  }};
}

macro_rules! w {
  ($o:expr, $v:expr) => {{
    $o.get_or_insert($v)
  }};
}

macro_rules! f {
  ($o:expr, $v:expr) => {{
    $o.unwrap_or_else($v)
  }};
}

macro_rules! expect {
  ($o:expr) => {
    $o.expect("value should not be `Option::None` after default apply")
  };
}

use devtool::DevtoolFlags;
use enum_tag::EnumTag;
use externals::ExternalsPresets;
use indexmap::IndexMap;
use rspack_core::{incremental::IncrementalPasses, ModuleType};
use rspack_core::{
  AssetParserDataUrl, AssetParserDataUrlOptions, AssetParserOptions, BoxPlugin, ByDependency,
  CacheOptions, ChunkLoading, ChunkLoadingType, CleanOptions, CompilerOptions, Context,
  CrossOriginLoading, CssAutoGeneratorOptions, CssAutoParserOptions, CssExportsConvention,
  CssGeneratorOptions, CssModuleGeneratorOptions, CssModuleParserOptions, CssParserOptions,
  DynamicImportMode, EntryDescription, Environment, ExperimentCacheOptions, Experiments,
  ExternalItem, ExternalType, Filename, FilenameTemplate, GeneratorOptions, GeneratorOptionsMap,
  JavascriptParserOptions, JavascriptParserOrder, JavascriptParserUrl, JsonParserOptions,
  LibraryName, LibraryNonUmdObject, LibraryOptions, LibraryType, MangleExportsOption, Mode,
  ModuleNoParseRules, ModuleOptions, ModuleRule, ModuleRuleEffect, Optimization, OutputOptions,
  ParseOption, ParserOptions, ParserOptionsMap, PathInfo, PublicPath, Resolve, RspackFuture,
  RuleSetCondition, RuleSetLogicalConditions, SideEffectOption, TrustedTypes, UsedExportsOption,
  WasmLoading, WasmLoadingType,
};
use rspack_hash::{HashDigest, HashFunction, HashSalt};
use rspack_paths::{AssertUtf8, Utf8PathBuf};
use rspack_regex::RspackRegex;
use rustc_hash::FxHashMap as HashMap;
use target::{get_targets_properties, TargetProperties};

pub trait Builder {
  type Item;
  fn builder() -> Self::Item;
}

impl Builder for CompilerOptions {
  type Item = CompilerOptionsBuilder;
  fn builder() -> Self::Item {
    CompilerOptionsBuilder::default()
  }
}

impl Builder for OutputOptions {
  type Item = OutputOptionsBuilder;
  fn builder() -> Self::Item {
    OutputOptionsBuilder::default()
  }
}

impl Builder for Optimization {
  type Item = OptimizationOptionsBuilder;
  fn builder() -> Self::Item {
    OptimizationOptionsBuilder::default()
  }
}

impl Builder for ModuleOptions {
  type Item = ModuleOptionsBuilder;
  fn builder() -> Self::Item {
    ModuleOptionsBuilder::default()
  }
}

impl Builder for Experiments {
  type Item = ExperimentsBuilder;
  fn builder() -> Self::Item {
    ExperimentsBuilder::default()
  }
}

/// Options of builtin plugins
///
/// The order of this list is strictly ordered with respect to `rspackOptionsApply`.
#[allow(unused, clippy::enum_variant_names)]
#[derive(Debug, EnumTag)]
#[repr(u8)]
pub(crate) enum BuiltinPluginOptions {
  // External handling plugins
  ExternalsPlugin((ExternalType, Vec<ExternalItem>)),
  NodeTargetPlugin,
  ElectronTargetPlugin(rspack_plugin_externals::ElectronTargetContext),
  HttpExternalsRspackPlugin((bool /* css */, bool /* web_async */)),

  // Chunk format and loading plugins
  ChunkPrefetchPreloadPlugin,
  CommonJsChunkFormatPlugin,
  ArrayPushCallbackChunkFormatPlugin,
  ModuleChunkFormatPlugin,
  EnableChunkLoadingPlugin(ChunkLoadingType),
  EnableWasmLoadingPlugin(WasmLoadingType),

  // Runtime and error handling
  RuntimeChunkPlugin(rspack_plugin_runtime_chunk::RuntimeChunkOptions),
  NoEmitOnErrorsPlugin,

  // DevTool plugins
  SourceMapDevToolPlugin(rspack_plugin_devtool::SourceMapDevToolPluginOptions),
  EvalSourceMapDevToolPlugin(rspack_plugin_devtool::SourceMapDevToolPluginOptions),
  EvalDevToolModulePlugin(rspack_plugin_devtool::EvalDevToolModulePluginOptions),

  // Core module plugins
  JavascriptModulesPlugin,
  JsonModulesPlugin,
  AssetModulesPlugin,
  AsyncWebAssemblyModulesPlugin,
  CssModulesPlugin,

  // Entry and runtime plugins
  EntryPlugin,
  RuntimePlugin,
  BundlerInfoRspackPlugin,

  // Core functionality plugins
  InferAsyncModulesPlugin,
  APIPlugin,
  DataUriPlugin,
  FileUriPlugin,

  // Optimization plugins
  EnsureChunkConditionsPlugin,
  MergeDuplicateChunksPlugin,
  SideEffectsFlagPlugin,
  FlagDependencyExportsPlugin,
  FlagDependencyUsagePlugin(bool),
  ModuleConcatenationPlugin,
  MangleExportsPlugin(bool),

  // Experiments
  LazyCompilationPlugin,

  // Output plugins
  EnableLibraryPlugin(LibraryType),
  SplitChunksPlugin,
  RemoveEmptyChunksPlugin,
  RealContentHashPlugin,

  // Module and chunk ID plugins
  NamedModuleIdsPlugin,
  NaturalModuleIdsPlugin,
  DeterministicModuleIdsPlugin,
  NaturalChunkIdsPlugin,
  NamedChunkIdsPlugin,
  DeterministicChunkIdsPlugin,
  OccurrenceChunkIdsPlugin(rspack_ids::OccurrenceChunkIdsPluginOptions),

  // Define and optimization plugins
  DefinePlugin(rspack_plugin_javascript::define_plugin::DefineValue),
  AnyMinimizerRspackPlugin(BoxPlugin),
  SizeLimitsPlugin,

  // Cache plugins
  MemoryCachePlugin,

  // Worker plugins
  WorkerPlugin,

  // Stats plugins
  DefaultStatsFactoryPlugin,
  DefaultStatsPresetPlugin,
  DefaultStatsPrinterPlugin,

  // Other core plugins
  ProgressPlugin,
  DynamicEntryPlugin,
  BannerPlugin,
  IgnorePlugin,
  FetchCompileAsyncWasmPlugin,
  HotModuleReplacementPlugin,
  LimitChunkCountPlugin,
  WebWorkerTemplatePlugin,
  RemoveDuplicateModulesPlugin,
  ShareRuntimePlugin,
  ContainerPlugin,
  ContainerReferencePlugin,
  ProvideSharedPlugin,
  ConsumeSharedPlugin,
  ModuleFederationRuntimePlugin,
  WarnCaseSensitiveModulesPlugin,
  ContextReplacementPlugin,
  DllEntryPlugin,
  DllReferenceAgencyPlugin,
  LibManifestPlugin,
  FlagAllModulesAsUsedPlugin,
  ProvidePlugin,

  // Rspack specific plugins
  CopyRspackPlugin,
  HtmlRspackPlugin,
  CssExtractRspackPlugin,
}

#[derive(Default, Debug)]
pub struct BuilderContext {
  plugins: Vec<BuiltinPluginOptions>,
}

/// Builder used to build [`CompilerOptions`]
#[derive(Debug, Default)]
pub struct CompilerOptionsBuilder {
  name: Option<String>,
  target: Option<Targets>,
  entry: IndexMap<String, EntryDescription>,
  externals: Option<Vec<ExternalItem>>,
  externals_type: Option<ExternalType>,
  externals_presets: Option<ExternalsPresets>,
  context: Option<Context>,
  cache: Option<CacheOptions>,
  mode: Option<Mode>,
  devtool: Option<Devtool>,
  profile: Option<bool>,
  bail: Option<bool>,
  experiments: Option<ExperimentsBuilder>,
  module: Option<ModuleOptionsBuilder>,
  output: Option<OutputOptionsBuilder>,
  optimization: Option<OptimizationOptionsBuilder>,
}

impl CompilerOptionsBuilder {
  pub fn name(&mut self, name: String) -> &mut Self {
    self.name = Some(name);
    self
  }

  pub fn target(&mut self, targets: Targets) -> &mut Self {
    self.target = Some(targets);
    self
  }

  pub fn entry(&mut self, entry_name: String, entry_description: EntryDescription) -> &mut Self {
    self.entry.insert(entry_name, entry_description);
    self
  }

  pub fn externals(&mut self, externals: ExternalItem) -> &mut Self {
    match &mut self.externals {
      Some(e) => e.push(externals),
      None => self.externals = Some(vec![externals]),
    }
    self
  }

  pub fn externals_type(&mut self, externals_type: ExternalType) -> &mut Self {
    self.externals_type = Some(externals_type);
    self
  }

  pub fn externals_presets(&mut self, externals_presets: ExternalsPresets) -> &mut Self {
    self.externals_presets = Some(externals_presets);
    self
  }

  pub fn context<V>(&mut self, context: V) -> &mut Self
  where
    V: Into<Context>,
  {
    self.context = Some(context.into());
    self
  }

  pub fn cache(&mut self, cache: CacheOptions) -> &mut Self {
    self.cache = Some(cache);
    self
  }

  pub fn devtool(&mut self, devtool: Devtool) -> &mut Self {
    self.devtool = Some(devtool);
    self
  }

  pub fn mode(&mut self, mode: Mode) -> &mut Self {
    self.mode = Some(mode);
    self
  }

  pub fn bail(&mut self, bail: bool) -> &mut Self {
    self.bail = Some(bail);
    self
  }

  pub fn profile(&mut self, profile: bool) -> &mut Self {
    self.profile = Some(profile);
    self
  }

  pub fn module<V>(&mut self, module: V) -> &mut Self
  where
    V: Into<ModuleOptionsBuilder>,
  {
    self.module = Some(module.into());
    self
  }

  pub fn output<V>(&mut self, output: V) -> &mut Self
  where
    V: Into<OutputOptionsBuilder>,
  {
    self.output = Some(output.into());
    self
  }

  pub fn optimization<V>(&mut self, optimization: V) -> &mut Self
  where
    V: Into<OptimizationOptionsBuilder>,
  {
    self.optimization = Some(optimization.into());
    self
  }

  pub fn experiments<V>(&mut self, experiments: V) -> &mut Self
  where
    V: Into<ExperimentsBuilder>,
  {
    self.experiments = Some(experiments.into());
    self
  }

  pub fn build(&mut self, builder_context: &mut BuilderContext) -> CompilerOptions {
    let name = self.name.take();
    let context = f!(self.context.take(), || {
      std::env::current_dir()
        .expect("`current_dir` should be available")
        .assert_utf8()
        .into()
    });

    // TODO: support browserlist default target
    let target = f!(self.target.take(), || vec!["web".to_string()]);
    let target_properties = get_targets_properties(&target, &context);

    let development = matches!(self.mode, Some(Mode::Development));
    let production = matches!(self.mode, Some(Mode::Production) | None);
    let mode = d!(self.mode.take(), Mode::Production);

    // TODO: support entry
    let devtool = f!(self.devtool.take(), || {
      if development {
        Devtool::Eval
      } else {
        Devtool::False
      }
    });

    let profile = d!(self.profile.take(), false);
    let bail = d!(self.bail.take(), false);
    let cache = d!(self.cache.take(), {
      if development {
        CacheOptions::Memory
      } else {
        CacheOptions::Disabled
      }
    });

    // apply experiments defaults
    let mut experiments_builder = f!(self.experiments.take(), Experiments::builder);
    let mut experiments = experiments_builder.build(builder_context, development, production);
    // Disable experiments cache if global cache is set to `Disabled`
    if matches!(cache, CacheOptions::Disabled) {
      experiments.cache = ExperimentCacheOptions::Disabled;
    }

    let async_web_assembly = expect!(experiments_builder.async_web_assembly);
    if async_web_assembly {
      builder_context
        .plugins
        .push(BuiltinPluginOptions::AsyncWebAssemblyModulesPlugin);
    }
    let css = expect!(experiments_builder.css);
    if css {
      builder_context
        .plugins
        .push(BuiltinPluginOptions::CssModulesPlugin);
    }
    let future_defaults = expect!(experiments_builder.future_defaults);
    let output_module = expect!(experiments_builder.output_module);

    // apply module defaults
    let module = f!(self.module.take(), ModuleOptions::builder).build(
      builder_context,
      async_web_assembly,
      css,
      &target_properties,
      &mode,
    );

    // apply output defaults
    let is_affected_by_browserslist = target.iter().any(|t| t.starts_with("browserslist"));
    let mut output_builder = f!(self.output.take(), OutputOptions::builder);
    let output = output_builder.build(
      builder_context,
      &context,
      output_module,
      Some(&target_properties),
      is_affected_by_browserslist,
      development,
      &self.entry,
      future_defaults,
    );

    // apply devtool plugin
    let devtool_flags = DevtoolFlags::from(devtool);
    if devtool_flags.source_map() {
      let hidden = devtool_flags.hidden();
      let inline = devtool_flags.inline();
      let eval_wrapped = devtool_flags.eval();
      let cheap = devtool_flags.cheap();
      let module_maps = devtool_flags.module();
      let no_sources = devtool_flags.nosources();

      let options = rspack_plugin_devtool::SourceMapDevToolPluginOptions {
        filename: (!inline).then_some(output.source_map_filename.as_str().to_string()),
        module_filename_template: output_builder
          .devtool_module_filename_template
          .map(|t| rspack_plugin_devtool::ModuleFilenameTemplate::String(t.as_str().to_string()))
          .clone(),
        append: hidden.then_some(rspack_plugin_devtool::Append::Disabled),
        columns: !cheap,
        fallback_module_filename_template: output_builder
          .devtool_fallback_module_filename_template
          .map(|t| rspack_plugin_devtool::ModuleFilenameTemplate::String(t.as_str().to_string()))
          .clone(),
        module: if module_maps { true } else { !cheap },
        namespace: output_builder.devtool_namespace.clone(),
        no_sources,
        file_context: None,
        public_path: None,
        source_root: None,
        test: None,
        include: None,
        exclude: None,
      };

      if eval_wrapped {
        builder_context
          .plugins
          .push(BuiltinPluginOptions::EvalSourceMapDevToolPlugin(options));
      } else {
        builder_context
          .plugins
          .push(BuiltinPluginOptions::SourceMapDevToolPlugin(options));
      }
    } else if devtool_flags.eval() {
      let options = rspack_plugin_devtool::EvalDevToolModulePluginOptions {
        module_filename_template: output_builder
          .devtool_module_filename_template
          .map(|t| rspack_plugin_devtool::ModuleFilenameTemplate::String(t.as_str().to_string()))
          .clone(),
        namespace: output_builder.devtool_namespace.clone(),
        source_url_comment: None,
      };
      builder_context
        .plugins
        .push(BuiltinPluginOptions::EvalDevToolModulePlugin(options));
    }

    // TODO: bundler info

    // apply externals presets defaults
    let externals_presets = self.externals_presets.get_or_insert_default();
    let tp = &target_properties;
    w!(externals_presets.node, tp.node());
    w!(externals_presets.electron, tp.electron());
    w!(
      externals_presets.electron_main,
      tp.electron() && tp.electron_main()
    );
    w!(
      externals_presets.electron_preload,
      tp.electron() && tp.electron_preload()
    );
    w!(
      externals_presets.electron_renderer,
      tp.electron() && tp.electron_renderer()
    );
    w!(externals_presets.nwjs, tp.nwjs());

    w!(self.externals_type, {
      if let Some(library) = &output.library {
        library.library_type.clone()
      } else if output.module {
        "module-import".to_string()
      } else {
        "var".to_string()
      }
    });

    // apply externals plugin
    if let Some(externals) = &mut self.externals {
      let externals = std::mem::take(externals);
      builder_context
        .plugins
        .push(BuiltinPluginOptions::ExternalsPlugin((
          expect!(self.externals_type.clone()),
          externals,
        )));
    }

    // apply externals presets plugin
    if externals_presets.node() {
      builder_context
        .plugins
        .push(BuiltinPluginOptions::NodeTargetPlugin);
    }

    use rspack_plugin_externals::ElectronTargetContext;

    if externals_presets.electron_main() {
      builder_context
        .plugins
        .push(BuiltinPluginOptions::ElectronTargetPlugin(
          ElectronTargetContext::Main,
        ));
    }
    if externals_presets.electron_preload() {
      builder_context
        .plugins
        .push(BuiltinPluginOptions::ElectronTargetPlugin(
          ElectronTargetContext::Preload,
        ));
    }
    if externals_presets.electron_renderer() {
      builder_context
        .plugins
        .push(BuiltinPluginOptions::ElectronTargetPlugin(
          ElectronTargetContext::Renderer,
        ));
    }
    if externals_presets.electron()
      && !externals_presets.electron_main()
      && !externals_presets.electron_preload()
      && !externals_presets.electron_renderer()
    {
      builder_context
        .plugins
        .push(BuiltinPluginOptions::ElectronTargetPlugin(
          ElectronTargetContext::None,
        ));
    }

    if externals_presets.nwjs() {
      builder_context
        .plugins
        .push(BuiltinPluginOptions::ExternalsPlugin((
          "node-commonjs".to_string(),
          vec!["nw.gui".to_string().into()],
        )));
    }

    if externals_presets.web() || externals_presets.web_async() || (externals_presets.node() && css)
    {
      builder_context
        .plugins
        .push(BuiltinPluginOptions::HttpExternalsRspackPlugin((
          css,
          externals_presets.web_async(),
        )));
    }

    // apply optimization defaults
    let optimization = f!(self.optimization.take(), Optimization::builder).build(
      builder_context,
      development,
      production,
      css,
    );

    // apply unconditional plugins
    builder_context
      .plugins
      .push(BuiltinPluginOptions::ChunkPrefetchPreloadPlugin);
    builder_context
      .plugins
      .push(BuiltinPluginOptions::JavascriptModulesPlugin);
    builder_context
      .plugins
      .push(BuiltinPluginOptions::JsonModulesPlugin);
    builder_context
      .plugins
      .push(BuiltinPluginOptions::AssetModulesPlugin);

    // TODO: entry

    builder_context
      .plugins
      .push(BuiltinPluginOptions::RuntimePlugin);
    builder_context
      .plugins
      .push(BuiltinPluginOptions::InferAsyncModulesPlugin);
    builder_context
      .plugins
      .push(BuiltinPluginOptions::APIPlugin);
    builder_context
      .plugins
      .push(BuiltinPluginOptions::DataUriPlugin);
    builder_context
      .plugins
      .push(BuiltinPluginOptions::FileUriPlugin);

    // TODO: options
    builder_context
      .plugins
      .push(BuiltinPluginOptions::WorkerPlugin);

    // TODO: stats plugins

    CompilerOptions {
      name,
      context,
      output,
      mode,
      resolve: Default::default(),
      resolve_loader: Default::default(),
      module,
      stats: Default::default(),
      cache,
      experiments,
      node: Default::default(),
      optimization,
      profile,
      amd: None,
      bail,
      __references: Default::default(),
    }
  }
}

/// Builder used to build [`ModuleOptions`]
#[derive(Debug, Default)]
pub struct ModuleOptionsBuilder {
  rules: Vec<ModuleRule>,
  parser: Option<ParserOptionsMap>,
  generator: Option<GeneratorOptionsMap>,
  no_parse: Option<ModuleNoParseRules>,
}

impl ModuleOptionsBuilder {
  pub fn rule(&mut self, rule: ModuleRule) -> &mut Self {
    self.rules.push(rule);
    self
  }

  pub fn rules(&mut self, mut rules: Vec<ModuleRule>) -> &mut Self {
    self.rules.append(&mut rules);
    self
  }

  pub fn parser(&mut self, parser: ParserOptionsMap) -> &mut Self {
    match &mut self.parser {
      Some(p) => p.extend(parser.clone()),
      None => self.parser = Some(parser),
    }
    self
  }

  pub fn generator(&mut self, generator: GeneratorOptionsMap) -> &mut Self {
    match &mut self.generator {
      Some(g) => g.extend(generator.clone()),
      None => self.generator = Some(generator),
    }
    self
  }

  pub fn no_parse(&mut self, no_parse: ModuleNoParseRules) -> &mut Self {
    self.no_parse = Some(no_parse);
    self
  }

  pub fn build(
    &mut self,
    _builder_context: &mut BuilderContext,
    async_web_assembly: bool,
    css: bool,
    target_properties: &TargetProperties,
    mode: &Mode,
  ) -> ModuleOptions {
    let parser = self.parser.get_or_insert(ParserOptionsMap::default());

    if !parser.contains_key("asset") {
      parser.insert(
        "asset".to_string(),
        ParserOptions::Asset(AssetParserOptions {
          data_url_condition: Some(AssetParserDataUrl::Options(AssetParserDataUrlOptions {
            max_size: Some(8096.0),
          })),
        }),
      );
    }

    if !parser.contains_key("javascript") {
      parser.insert(
        "javascript".to_string(),
        ParserOptions::Javascript(JavascriptParserOptions {
          dynamic_import_mode: Some(DynamicImportMode::Lazy),
          dynamic_import_preload: Some(JavascriptParserOrder::Disable),
          dynamic_import_prefetch: Some(JavascriptParserOrder::Disable),
          dynamic_import_fetch_priority: None,
          url: Some(JavascriptParserUrl::Enable),
          expr_context_critical: Some(true),
          wrapped_context_critical: Some(false),
          wrapped_context_reg_exp: Some(RspackRegex::new(".*").expect("should initialize `Regex`")),
          strict_export_presence: Some(false),
          worker: Some(vec!["...".to_string()]),
          import_meta: Some(true),
          require_as_expression: Some(true),
          require_dynamic: Some(true),
          require_resolve: Some(true),
          import_dynamic: Some(true),
          ..Default::default()
        }),
      );
    }

    if !parser.contains_key("json") {
      parser.insert(
        "json".to_string(),
        ParserOptions::Json(JsonParserOptions {
          exports_depth: if matches!(mode, Mode::Development) {
            Some(1)
          } else {
            Some(u32::MAX)
          },
          parse: ParseOption::None,
        }),
      );
    }

    if css {
      let generator = self.generator.get_or_insert(GeneratorOptionsMap::default());

      let css_parser_options = ParserOptions::Css(CssParserOptions {
        named_exports: Some(true),
      });
      parser.insert("css".to_string(), css_parser_options.clone());

      let css_auto_parser_options = ParserOptions::CssAuto(CssAutoParserOptions {
        named_exports: Some(true),
      });
      parser.insert("css/auto".to_string(), css_auto_parser_options);

      let css_module_parser_options = ParserOptions::CssModule(CssModuleParserOptions {
        named_exports: Some(true),
      });
      parser.insert("css/module".to_string(), css_module_parser_options);

      // CSS generator options
      let exports_only = !target_properties.document();

      generator.insert(
        "css".to_string(),
        GeneratorOptions::Css(CssGeneratorOptions {
          exports_only: Some(exports_only),
          es_module: Some(true),
        }),
      );

      generator.insert(
        "css/auto".to_string(),
        GeneratorOptions::CssAuto(CssAutoGeneratorOptions {
          exports_only: Some(exports_only),
          exports_convention: Some(CssExportsConvention::default()),
          local_ident_name: Some("[uniqueName]-[id]-[local]".into()),

          es_module: Some(true),
        }),
      );

      generator.insert(
        "css/module".to_string(),
        GeneratorOptions::CssModule(CssModuleGeneratorOptions {
          exports_only: Some(exports_only),
          exports_convention: Some(CssExportsConvention::default()),
          local_ident_name: Some("[uniqueName]-[id]-[local]".into()),
          es_module: Some(true),
        }),
      );
    }

    let default_rules = default_rules(async_web_assembly, css);

    ModuleOptions {
      rules: vec![
        ModuleRule {
          rules: Some(default_rules),
          ..Default::default()
        },
        ModuleRule {
          rules: Some(std::mem::take(&mut self.rules)),
          ..Default::default()
        },
      ],
      parser: self.parser.take(),
      generator: self.generator.take(),
      no_parse: self.no_parse.take(),
    }
  }
}

fn default_rules(async_web_assembly: bool, css: bool) -> Vec<ModuleRule> {
  let mut rules = vec![
    // application/node
    ModuleRule {
      mimetype: Some(RuleSetCondition::String("application/node".into()).into()),
      effect: ModuleRuleEffect {
        r#type: Some(ModuleType::JsAuto),
        ..Default::default()
      },
      ..Default::default()
    },
    // .json
    ModuleRule {
      test: Some(RuleSetCondition::Regexp(
        RspackRegex::new(r"\.json$").expect("should initialize `Regex`"),
      )),
      effect: ModuleRuleEffect {
        r#type: Some(ModuleType::Json),
        ..Default::default()
      },
      ..Default::default()
    },
    // application/json
    ModuleRule {
      mimetype: Some(RuleSetCondition::String("application/json".into()).into()),
      effect: ModuleRuleEffect {
        r#type: Some(ModuleType::Json),
        ..Default::default()
      },
      ..Default::default()
    },
    // .mjs
    ModuleRule {
      test: Some(RuleSetCondition::Regexp(
        RspackRegex::new(r"\.mjs$").expect("should initialize `Regex`"),
      )),
      effect: ModuleRuleEffect {
        r#type: Some(ModuleType::JsEsm),
        resolve: Some(Resolve {
          by_dependency: Some(ByDependency::from_iter([(
            "esm".into(),
            Resolve {
              fully_specified: Some(true),
              ..Default::default()
            },
          )])),
          ..Default::default()
        }),
        ..Default::default()
      },
      ..Default::default()
    },
    // .js with type:module
    ModuleRule {
      test: Some(RuleSetCondition::Regexp(
        RspackRegex::new(r"\.js$").expect("should initialize `Regex`"),
      )),
      description_data: Some(HashMap::from_iter([(
        "type".into(),
        RuleSetCondition::String("module".into()).into(),
      )])),
      effect: ModuleRuleEffect {
        r#type: Some(ModuleType::JsEsm),
        resolve: Some(Resolve {
          by_dependency: Some(ByDependency::from_iter([(
            "esm".into(),
            Resolve {
              fully_specified: Some(true),
              ..Default::default()
            },
          )])),
          ..Default::default()
        }),
        ..Default::default()
      },
      ..Default::default()
    },
    // .cjs
    ModuleRule {
      test: Some(RuleSetCondition::Regexp(
        RspackRegex::new(r"\.cjs$").expect("should initialize `Regex`"),
      )),
      effect: ModuleRuleEffect {
        r#type: Some(ModuleType::JsDynamic),
        ..Default::default()
      },
      ..Default::default()
    },
    // .js with type:commonjs
    ModuleRule {
      test: Some(RuleSetCondition::Regexp(
        RspackRegex::new(r"\.js$").expect("should initialize `Regex`"),
      )),
      description_data: Some(HashMap::from_iter([(
        "type".into(),
        RuleSetCondition::String("commonjs".into()).into(),
      )])),
      effect: ModuleRuleEffect {
        r#type: Some(ModuleType::JsDynamic),
        ..Default::default()
      },
      ..Default::default()
    },
    // text/javascript or application/javascript
    ModuleRule {
      mimetype: Some(
        RuleSetCondition::Logical(Box::new(RuleSetLogicalConditions {
          or: Some(vec![
            RuleSetCondition::String("text/javascript".into()),
            RuleSetCondition::String("application/javascript".into()),
          ]),
          ..Default::default()
        }))
        .into(),
      ),
      effect: ModuleRuleEffect {
        r#type: Some(ModuleType::JsEsm),
        resolve: Some(Resolve {
          by_dependency: Some(ByDependency::from_iter([(
            "esm".into(),
            Resolve {
              fully_specified: Some(true),
              ..Default::default()
            },
          )])),
          ..Default::default()
        }),
        ..Default::default()
      },
      ..Default::default()
    },
  ];

  // Add WebAssembly rules if enabled
  if async_web_assembly {
    rules.extend(vec![
      ModuleRule {
        test: Some(RuleSetCondition::Regexp(
          RspackRegex::new(r"\.wasm$").expect("should initialize `Regex`"),
        )),
        effect: ModuleRuleEffect {
          r#type: Some(ModuleType::WasmAsync),
          ..Default::default()
        },
        rules: Some(vec![ModuleRule {
          description_data: Some(HashMap::from_iter([(
            "type".into(),
            RuleSetCondition::String("module".into()).into(),
          )])),
          effect: ModuleRuleEffect {
            resolve: Some(Resolve {
              fully_specified: Some(true),
              ..Default::default()
            }),
            ..Default::default()
          },
          ..Default::default()
        }]),
        ..Default::default()
      },
      ModuleRule {
        mimetype: Some(RuleSetCondition::String("application/wasm".into()).into()),
        effect: ModuleRuleEffect {
          r#type: Some(ModuleType::WasmAsync),
          ..Default::default()
        },
        rules: Some(vec![ModuleRule {
          description_data: Some(HashMap::from_iter([(
            "type".into(),
            RuleSetCondition::String("module".into()).into(),
          )])),
          effect: ModuleRuleEffect {
            resolve: Some(Resolve {
              fully_specified: Some(true),
              ..Default::default()
            }),
            ..Default::default()
          },
          ..Default::default()
        }]),
        ..Default::default()
      },
    ]);
  }

  // Add CSS rules if enabled
  if css {
    let resolve = Resolve {
      fully_specified: Some(true),
      prefer_relative: Some(true),
      ..Default::default()
    };

    rules.extend(vec![
      ModuleRule {
        test: Some(RuleSetCondition::Regexp(
          RspackRegex::new(r"\.css$").expect("should initialize `Regex`"),
        )),
        effect: ModuleRuleEffect {
          r#type: Some(ModuleType::CssAuto),
          resolve: Some(resolve.clone()),
          ..Default::default()
        },
        ..Default::default()
      },
      ModuleRule {
        mimetype: Some(RuleSetCondition::String("text/css+module".into()).into()),
        effect: ModuleRuleEffect {
          r#type: Some(ModuleType::CssModule),
          resolve: Some(resolve.clone()),
          ..Default::default()
        },
        ..Default::default()
      },
      ModuleRule {
        mimetype: Some(RuleSetCondition::String("text/css".into()).into()),
        effect: ModuleRuleEffect {
          r#type: Some(ModuleType::Css),
          resolve: Some(resolve),
          ..Default::default()
        },
        ..Default::default()
      },
    ]);
  }

  // Add URL dependency rules
  rules.extend(vec![
    ModuleRule {
      dependency: Some(RuleSetCondition::String("url".into())),
      one_of: Some(vec![
        ModuleRule {
          scheme: Some(
            RuleSetCondition::Regexp(
              RspackRegex::new("^data$").expect("should initialize `Regex`"),
            )
            .into(),
          ),
          effect: ModuleRuleEffect {
            r#type: Some(ModuleType::AssetInline),
            ..Default::default()
          },
          ..Default::default()
        },
        ModuleRule {
          effect: ModuleRuleEffect {
            r#type: Some(ModuleType::AssetResource),
            ..Default::default()
          },
          ..Default::default()
        },
      ]),
      ..Default::default()
    },
    ModuleRule {
      with: Some(HashMap::from_iter([(
        "type".into(),
        RuleSetCondition::String("json".into()).into(),
      )])),
      effect: ModuleRuleEffect {
        r#type: Some(ModuleType::Json),
        ..Default::default()
      },
      ..Default::default()
    },
  ]);

  rules
}

/// Builder used to build [`OutputOptions`]
#[derive(Debug, Default)]
pub struct OutputOptionsBuilder {
  path: Option<Utf8PathBuf>,
  pathinfo: Option<PathInfo>,
  clean: Option<CleanOptions>,
  public_path: Option<PublicPath>,
  asset_module_filename: Option<Filename>,
  wasm_loading: Option<WasmLoading>,
  webassembly_module_filename: Option<FilenameTemplate>,
  unique_name: Option<String>,
  chunk_loading: Option<ChunkLoading>,
  chunk_loading_global: Option<String>,
  chunk_load_timeout: Option<u32>,
  chunk_format: Option<String>,
  charset: Option<bool>,
  filename: Option<Filename>,
  chunk_filename: Option<Filename>,
  cross_origin_loading: Option<CrossOriginLoading>,
  css_filename: Option<Filename>,
  css_chunk_filename: Option<Filename>,
  hot_update_main_filename: Option<FilenameTemplate>,
  hot_update_chunk_filename: Option<FilenameTemplate>,
  hot_update_global: Option<String>,
  library: Option<LibraryOptions>,
  enabled_library_types: Option<Vec<LibraryType>>,
  enabled_chunk_loading_types: Option<Vec<ChunkLoadingType>>,
  enabled_wasm_loading_types: Option<Vec<WasmLoadingType>>,
  strict_module_error_handling: Option<bool>,
  global_object: Option<String>,
  import_function_name: Option<String>,
  import_meta_name: Option<String>,
  iife: Option<bool>,
  module: Option<bool>,
  trusted_types: Option<TrustedTypes>,
  source_map_filename: Option<FilenameTemplate>,
  hash_function: Option<HashFunction>,
  hash_digest: Option<HashDigest>,
  hash_digest_length: Option<usize>,
  hash_salt: Option<HashSalt>,
  async_chunks: Option<bool>,
  worker_chunk_loading: Option<ChunkLoading>,
  worker_wasm_loading: Option<WasmLoading>,
  worker_public_path: Option<String>,
  script_type: Option<String>,
  devtool_namespace: Option<String>,
  devtool_module_filename_template: Option<FilenameTemplate>,
  devtool_fallback_module_filename_template: Option<FilenameTemplate>,
  environment: Option<Environment>,
  compare_before_emit: Option<bool>,
}

impl OutputOptionsBuilder {
  pub fn path<V>(&mut self, path: V) -> &mut Self
  where
    V: Into<Utf8PathBuf>,
  {
    self.path = Some(path.into());
    self
  }

  pub fn pathinfo(&mut self, pathinfo: PathInfo) -> &mut Self {
    self.pathinfo = Some(pathinfo);
    self
  }

  pub fn clean(&mut self, clean: CleanOptions) -> &mut Self {
    self.clean = Some(clean);
    self
  }

  pub fn public_path(&mut self, public_path: PublicPath) -> &mut Self {
    self.public_path = Some(public_path);
    self
  }

  pub fn asset_module_filename(&mut self, filename: Filename) -> &mut Self {
    self.asset_module_filename = Some(filename);
    self
  }

  pub fn wasm_loading(&mut self, loading: WasmLoading) -> &mut Self {
    self.wasm_loading = Some(loading);
    self
  }

  pub fn webassembly_module_filename(&mut self, filename: FilenameTemplate) -> &mut Self {
    self.webassembly_module_filename = Some(filename);
    self
  }

  pub fn unique_name(&mut self, name: String) -> &mut Self {
    self.unique_name = Some(name);
    self
  }

  pub fn chunk_loading(&mut self, loading: ChunkLoading) -> &mut Self {
    self.chunk_loading = Some(loading);
    self
  }

  pub fn chunk_loading_global(&mut self, global: String) -> &mut Self {
    self.chunk_loading_global = Some(global);
    self
  }

  pub fn chunk_load_timeout(&mut self, timeout: u32) -> &mut Self {
    self.chunk_load_timeout = Some(timeout);
    self
  }

  pub fn chunk_format(&mut self, chunk_format: String) -> &mut Self {
    self.chunk_format = Some(chunk_format);
    self
  }

  pub fn charset(&mut self, charset: bool) -> &mut Self {
    self.charset = Some(charset);
    self
  }

  pub fn filename(&mut self, filename: Filename) -> &mut Self {
    self.filename = Some(filename);
    self
  }

  pub fn chunk_filename(&mut self, filename: Filename) -> &mut Self {
    self.chunk_filename = Some(filename);
    self
  }

  pub fn cross_origin_loading(&mut self, loading: CrossOriginLoading) -> &mut Self {
    self.cross_origin_loading = Some(loading);
    self
  }

  pub fn css_filename(&mut self, filename: Filename) -> &mut Self {
    self.css_filename = Some(filename);
    self
  }

  pub fn css_chunk_filename(&mut self, filename: Filename) -> &mut Self {
    self.css_chunk_filename = Some(filename);
    self
  }

  pub fn hot_update_main_filename(&mut self, filename: FilenameTemplate) -> &mut Self {
    self.hot_update_main_filename = Some(filename);
    self
  }

  pub fn hot_update_chunk_filename(&mut self, filename: FilenameTemplate) -> &mut Self {
    self.hot_update_chunk_filename = Some(filename);
    self
  }

  pub fn hot_update_global(&mut self, global: String) -> &mut Self {
    self.hot_update_global = Some(global);
    self
  }

  pub fn library(&mut self, library: LibraryOptions) -> &mut Self {
    self.library = Some(library);
    self
  }

  pub fn enabled_library_types(&mut self, types: Vec<LibraryType>) -> &mut Self {
    self.enabled_library_types = Some(types);
    self
  }

  pub fn enabled_chunk_loading_types(&mut self, types: Vec<ChunkLoadingType>) -> &mut Self {
    self.enabled_chunk_loading_types = Some(types);
    self
  }

  pub fn enabled_wasm_loading_types(&mut self, types: Vec<WasmLoadingType>) -> &mut Self {
    self.enabled_wasm_loading_types = Some(types);
    self
  }

  pub fn strict_module_error_handling(&mut self, strict: bool) -> &mut Self {
    self.strict_module_error_handling = Some(strict);
    self
  }

  pub fn global_object(&mut self, object: String) -> &mut Self {
    self.global_object = Some(object);
    self
  }

  pub fn import_function_name(&mut self, name: String) -> &mut Self {
    self.import_function_name = Some(name);
    self
  }

  pub fn import_meta_name(&mut self, name: String) -> &mut Self {
    self.import_meta_name = Some(name);
    self
  }

  pub fn iife(&mut self, iife: bool) -> &mut Self {
    self.iife = Some(iife);
    self
  }

  pub fn module(&mut self, module: bool) -> &mut Self {
    self.module = Some(module);
    self
  }

  pub fn trusted_types(&mut self, trusted_types: TrustedTypes) -> &mut Self {
    self.trusted_types = Some(trusted_types);
    self
  }

  pub fn source_map_filename(&mut self, filename: FilenameTemplate) -> &mut Self {
    self.source_map_filename = Some(filename);
    self
  }

  pub fn hash_function(&mut self, function: HashFunction) -> &mut Self {
    self.hash_function = Some(function);
    self
  }

  pub fn hash_digest(&mut self, digest: HashDigest) -> &mut Self {
    self.hash_digest = Some(digest);
    self
  }

  pub fn hash_digest_length(&mut self, length: usize) -> &mut Self {
    self.hash_digest_length = Some(length);
    self
  }

  pub fn hash_salt(&mut self, salt: HashSalt) -> &mut Self {
    self.hash_salt = Some(salt);
    self
  }

  pub fn async_chunks(&mut self, async_chunks: bool) -> &mut Self {
    self.async_chunks = Some(async_chunks);
    self
  }

  pub fn worker_chunk_loading(&mut self, loading: ChunkLoading) -> &mut Self {
    self.worker_chunk_loading = Some(loading);
    self
  }

  pub fn worker_wasm_loading(&mut self, loading: WasmLoading) -> &mut Self {
    self.worker_wasm_loading = Some(loading);
    self
  }

  pub fn worker_public_path(&mut self, path: String) -> &mut Self {
    self.worker_public_path = Some(path);
    self
  }

  pub fn script_type(&mut self, script_type: String) -> &mut Self {
    self.script_type = Some(script_type);
    self
  }

  pub fn devtool_namespace(&mut self, namespace: String) -> &mut Self {
    self.devtool_namespace = Some(namespace);
    self
  }

  pub fn devtool_module_filename_template(&mut self, filename: FilenameTemplate) -> &mut Self {
    self.devtool_module_filename_template = Some(filename);
    self
  }

  pub fn devtool_fallback_module_filename_template(
    &mut self,
    filename: FilenameTemplate,
  ) -> &mut Self {
    self.devtool_fallback_module_filename_template = Some(filename);
    self
  }
  pub fn environment(&mut self, environment: Environment) -> &mut Self {
    self.environment = Some(environment);
    self
  }

  pub fn compare_before_emit(&mut self, compare: bool) -> &mut Self {
    self.compare_before_emit = Some(compare);
    self
  }

  #[allow(clippy::too_many_arguments, clippy::fn_params_excessive_bools)]
  pub fn build(
    &mut self,
    builder_context: &mut BuilderContext,
    context: &Context,
    output_module: bool,
    target_properties: Option<&TargetProperties>,
    is_affected_by_browserslist: bool,
    development: bool,
    _entry: &IndexMap<String, EntryDescription>,
    _future_defaults: bool,
  ) -> OutputOptions {
    let tp = target_properties;

    let path = f!(self.path.take(), || { context.as_path().join("dist") });

    let pathinfo = f!(self.pathinfo.take(), || {
      if development {
        PathInfo::Bool(true)
      } else {
        PathInfo::Bool(false)
      }
    });

    let clean = d!(self.clean.take(), CleanOptions::CleanAll(false));

    let public_path = f!(self.public_path.take(), || {
      if tp.is_some_and(|t| t.document() || t.import_scripts()) {
        PublicPath::Auto
      } else {
        PublicPath::Filename("".into())
      }
    });

    let asset_module_filename = f!(self.asset_module_filename.take(), || {
      "[hash][ext][query]".into()
    });

    let filename = f!(self.filename.take(), || {
      if output_module {
        "[name].mjs".into()
      } else {
        "[name].js".into()
      }
    });

    let chunk_filename = f!(self.chunk_filename.take(), || {
      // Get template string from filename if it's not a function
      if let Some(template) = filename.template() {
        let has_name = template.contains("[name]");
        let has_id = template.contains("[id]");
        let has_chunk_hash = template.contains("[chunkhash]");
        let has_content_hash = template.contains("[contenthash]");

        // Anything changing depending on chunk is fine
        if has_chunk_hash || has_content_hash || has_name || has_id {
          filename.clone()
        } else {
          // Otherwise prefix "[id]." in front of the basename to make it changing
          let new_template = regex::Regex::new(r"(^|\/)([^/]*(?:\?|$))")
            .expect("should initialize `Regex`")
            .replace(template, "$1[id].$2")
            .into_owned();
          Filename::from(new_template)
        }
      } else {
        // If filename is a function, use default
        "[id].js".into()
      }
    });

    let css_filename = f!(self.css_filename.take(), || {
      if let Some(template) = filename.template() {
        let new_template = regex::Regex::new(r"\.[mc]?js(\?|$)")
          .expect("should initialize `Regex`")
          .replace(template, ".css$1")
          .into_owned();
        Filename::from(new_template)
      } else {
        "[id].css".into()
      }
    });

    let css_chunk_filename = f!(self.css_chunk_filename.take(), || {
      if let Some(template) = chunk_filename.template() {
        let new_template = regex::Regex::new(r"\.[mc]?js(\?|$)")
          .expect("should initialize `Regex`")
          .replace(template, ".css$1")
          .into_owned();
        Filename::from(new_template)
      } else {
        "[id].css".into()
      }
    });

    let hot_update_chunk_filename = f!(self.hot_update_chunk_filename.take(), || {
      format!(
        "[id].[fullhash].hot-update.{}",
        if output_module { "mjs" } else { "js" }
      )
      .into()
    });

    let hot_update_main_filename = f!(self.hot_update_main_filename.take(), || {
      "[runtime].[fullhash].hot-update.json".into()
    });

    // Generate unique name from library name or package.json
    let unique_name = f!(self.unique_name.take(), || {
      if let Some(library) = &self.library {
        if let Some(name) = &library.name {
          let library_name = match name {
            LibraryName::NonUmdObject(LibraryNonUmdObject::String(s)) => s.clone(),
            LibraryName::NonUmdObject(LibraryNonUmdObject::Array(arr)) => arr.join("."),
            LibraryName::UmdObject(obj) => {
              obj.root.as_ref().map(|r| r.join(".")).unwrap_or_default()
            }
          };

          // Clean up library name using regex
          let re = regex::Regex::new(
            r"^\[(\\*[\w:]+\\*)\](\.)|(\.)\[(\\*[\w:]+\\*)\](\.|\z)|\[(\\*[\w:]+\\*)\]",
          )
          .expect("failed to create regex");

          let cleaned_name = re.replace_all(&library_name, |caps: &regex::Captures| {
            let content = caps
              .get(1)
              .or_else(|| caps.get(4))
              .or_else(|| caps.get(6))
              .map_or("", |m| m.as_str());

            if content.starts_with('\\') && content.ends_with('\\') {
              format!(
                "{}{}{}",
                caps.get(3).map_or("", |_| "."),
                format_args!("[{}]", &content[1..content.len() - 1]),
                caps.get(2).map_or("", |_| ".")
              )
            } else {
              String::new()
            }
          });

          if !cleaned_name.is_empty() {
            return cleaned_name.into_owned();
          }
        }
      }

      // Try reading from package.json
      let pkg_path = path.join("package.json");
      if let Ok(pkg_content) = std::fs::read_to_string(pkg_path) {
        if let Ok(pkg_json) = serde_json::from_str::<serde_json::Value>(&pkg_content) {
          if let Some(name) = pkg_json.get("name").and_then(|n| n.as_str()) {
            return name.to_string();
          }
        }
      }
      String::new()
    });

    let chunk_loading_global = f!(self.chunk_loading_global.take(), || {
      format!("webpackChunk{}", rspack_core::to_identifier(&unique_name))
    });

    let hot_update_global = f!(self.hot_update_global.take(), || {
      format!(
        "webpackHotUpdate{}",
        rspack_core::to_identifier(&unique_name)
      )
    });

    // TODO: do not panic
    let chunk_format = f!(self.chunk_format.take(), || {
      if let Some(tp) = tp {
        let help_message = if is_affected_by_browserslist {
          "Make sure that your 'browserslist' includes only platforms that support these features or select an appropriate 'target' to allow selecting a chunk format by default. Alternatively specify the 'output.chunkFormat' directly."
        } else {
          "Select an appropriate 'target' to allow selecting one by default, or specify the 'output.chunkFormat' directly."
        };

        if output_module {
          if tp.dynamic_import() {
            "module".to_string()
          } else if tp.document() {
            "array-push".to_string()
          } else {
            panic!("For the selected environment is no default ESM chunk format available:\nESM exports can be chosen when 'import()' is available.\nJSONP Array push can be chosen when 'document' is available.\n{help_message}");
          }
        } else if tp.document() {
          "array-push".to_string()
        } else if tp.require() || tp.node_builtins() {
          "commonjs".to_string()
        } else if tp.import_scripts() {
          "array-push".to_string()
        } else {
          panic!("For the selected environment is no default script chunk format available:\nJSONP Array push can be chosen when 'document' or 'importScripts' is available.\nCommonJs exports can be chosen when 'require' or node builtins are available.\n{help_message}");
        }
      } else {
        panic!("Chunk format can't be selected by default when no target is specified");
      }
    });

    match &*chunk_format {
      "array-push" => {
        builder_context
          .plugins
          .push(BuiltinPluginOptions::ArrayPushCallbackChunkFormatPlugin);
      }
      "commonjs" => {
        builder_context
          .plugins
          .push(BuiltinPluginOptions::CommonJsChunkFormatPlugin);
      }
      "module" => {
        builder_context
          .plugins
          .push(BuiltinPluginOptions::ModuleChunkFormatPlugin);
      }
      _ => {}
    }

    let chunk_loading = f!(self.chunk_loading.take(), || {
      if let Some(tp) = tp {
        match &*chunk_format {
          "array-push" => {
            if tp.document() {
              ChunkLoading::Enable(ChunkLoadingType::Jsonp)
            } else if tp.import_scripts() {
              ChunkLoading::Enable(ChunkLoadingType::ImportScripts)
            } else {
              ChunkLoading::Disable
            }
          }
          "commonjs" => {
            if tp.require() {
              ChunkLoading::Enable(ChunkLoadingType::Require)
            } else if tp.node_builtins() {
              ChunkLoading::Enable(ChunkLoadingType::AsyncNode)
            } else {
              ChunkLoading::Disable
            }
          }
          "module" => {
            if tp.dynamic_import() {
              ChunkLoading::Enable(ChunkLoadingType::Import)
            } else {
              ChunkLoading::Disable
            }
          }
          _ => ChunkLoading::Disable,
        }
      } else {
        ChunkLoading::Disable
      }
    });

    let worker_chunk_loading = f!(self.worker_chunk_loading.take(), || {
      if let Some(tp) = tp {
        match &*chunk_format {
          "array-push" => {
            if tp.import_scripts_in_worker() {
              ChunkLoading::Enable(ChunkLoadingType::ImportScripts)
            } else {
              ChunkLoading::Disable
            }
          }
          "commonjs" => {
            if tp.require() {
              ChunkLoading::Enable(ChunkLoadingType::Require)
            } else if tp.node_builtins() {
              ChunkLoading::Enable(ChunkLoadingType::AsyncNode)
            } else {
              ChunkLoading::Disable
            }
          }
          "module" => {
            if tp.dynamic_import_in_worker() {
              ChunkLoading::Enable(ChunkLoadingType::Import)
            } else {
              ChunkLoading::Disable
            }
          }
          _ => ChunkLoading::Disable,
        }
      } else {
        ChunkLoading::Disable
      }
    });

    let wasm_loading = f!(self.wasm_loading.take(), || {
      if let Some(tp) = tp {
        if tp.fetch_wasm() {
          WasmLoading::Enable(WasmLoadingType::Fetch)
        } else if tp.node_builtins() {
          if output_module {
            WasmLoading::Enable(WasmLoadingType::AsyncNodeModule)
          } else {
            WasmLoading::Enable(WasmLoadingType::AsyncNode)
          }
        } else {
          WasmLoading::Disable
        }
      } else {
        WasmLoading::Disable
      }
    });

    let worker_wasm_loading = f!(self.worker_wasm_loading.take(), || wasm_loading.clone());

    let global_object = f!(self.global_object.take(), || {
      if let Some(tp) = tp {
        if tp.global() {
          "global".into()
        } else if tp.global_this() {
          "globalThis".into()
        } else {
          "self".into()
        }
      } else {
        "self".into()
      }
    });

    let enabled_library_types = f!(self.enabled_library_types.take(), || {
      let mut enabled_library_types = vec![];
      if let Some(library) = &self.library {
        enabled_library_types.push(library.library_type.clone());
      }
      // TODO: support entry
      enabled_library_types
    });

    for ty in enabled_library_types.iter() {
      builder_context
        .plugins
        .push(BuiltinPluginOptions::EnableLibraryPlugin(ty.clone()));
    }

    let enabled_chunk_loading_types = f!(self.enabled_chunk_loading_types.take(), || {
      let mut enabled_chunk_loading_types = vec![];
      if let ChunkLoading::Enable(ty) = chunk_loading {
        enabled_chunk_loading_types.push(ty);
      }
      if let ChunkLoading::Enable(ty) = worker_chunk_loading {
        enabled_chunk_loading_types.push(ty);
      }

      // TODO: support entry
      enabled_chunk_loading_types
    });

    for ty in enabled_chunk_loading_types.iter() {
      builder_context
        .plugins
        .push(BuiltinPluginOptions::EnableChunkLoadingPlugin(*ty));
    }

    let enabled_wasm_loading_types = f!(self.enabled_wasm_loading_types.take(), || {
      let mut enabled_wasm_loading_types = vec![];
      if let WasmLoading::Enable(ty) = wasm_loading {
        enabled_wasm_loading_types.push(ty);
      }
      if let WasmLoading::Enable(ty) = worker_wasm_loading {
        enabled_wasm_loading_types.push(ty);
      }
      // TODO: support entry
      enabled_wasm_loading_types
    });

    for ty in enabled_wasm_loading_types.iter() {
      builder_context
        .plugins
        .push(BuiltinPluginOptions::EnableWasmLoadingPlugin(*ty));
    }

    let environment = Environment {
      r#const: tp.and_then(|t| t.r#const),
      arrow_function: tp.and_then(|t| t.arrow_function),
      node_prefix_for_core_modules: tp.and_then(|t| t.node_prefix_for_core_modules),
    };

    OutputOptions {
      path,
      pathinfo,
      clean,
      asset_module_filename,
      public_path,
      wasm_loading,
      webassembly_module_filename: self
        .webassembly_module_filename
        .take()
        .unwrap_or_else(|| "[hash].module.wasm".into()),
      unique_name,
      chunk_loading,
      chunk_loading_global,
      chunk_load_timeout: self.chunk_load_timeout.take().unwrap_or(120_000),
      charset: self.charset.take().unwrap_or(true),
      filename,
      chunk_filename,
      cross_origin_loading: self
        .cross_origin_loading
        .take()
        .unwrap_or(CrossOriginLoading::Disable),
      css_filename,
      css_chunk_filename,
      hot_update_main_filename,
      hot_update_chunk_filename,
      hot_update_global,
      library: self.library.take(),
      enabled_library_types: Some(enabled_library_types),
      strict_module_error_handling: self.strict_module_error_handling.take().unwrap_or(false),
      global_object,
      import_function_name: self
        .import_function_name
        .take()
        .unwrap_or_else(|| "import".into()),
      import_meta_name: self
        .import_meta_name
        .take()
        .unwrap_or_else(|| "import.meta".into()),
      iife: self.iife.take().unwrap_or(!output_module),
      module: self.module.take().unwrap_or(output_module),
      trusted_types: self.trusted_types.take(),
      source_map_filename: self
        .source_map_filename
        .take()
        .unwrap_or_else(|| "[file].map[query]".into()),
      hash_function: self.hash_function.take().unwrap_or(HashFunction::Xxhash64),
      hash_digest: self.hash_digest.take().unwrap_or(HashDigest::Hex),
      hash_digest_length: self.hash_digest_length.take().unwrap_or(16),
      hash_salt: match self.hash_salt.take() {
        Some(salt) => salt,
        None => HashSalt::None,
      },
      async_chunks: self.async_chunks.take().unwrap_or(true),
      worker_chunk_loading,
      worker_wasm_loading,
      worker_public_path: self.worker_public_path.take().unwrap_or_default(),
      script_type: self.script_type.take().unwrap_or_else(|| {
        if output_module {
          "module".into()
        } else {
          String::new()
        }
      }),
      environment,
      compare_before_emit: self.compare_before_emit.take().unwrap_or(true),
    }
  }
}

/// Builder used to build options for optimization plugins
#[derive(Debug, Default)]
pub struct OptimizationOptionsBuilder {
  remove_available_modules: Option<bool>,
  remove_empty_chunks: Option<bool>,
  merge_duplicate_chunks: Option<bool>,
  module_ids: Option<String>,
  chunk_ids: Option<String>,
  minimize: Option<bool>,
  minimizer: Option<Vec<BuiltinPluginOptions>>,
  side_effects: Option<SideEffectOption>,
  provided_exports: Option<bool>,
  used_exports: Option<UsedExportsOption>,
  inner_graph: Option<bool>,
  mangle_exports: Option<MangleExportsOption>,
  concatenate_modules: Option<bool>,
  real_content_hash: Option<bool>,
  avoid_entry_iife: Option<bool>,
  node_env: Option<String>,
  emit_on_errors: Option<bool>,
  runtime_chunk: Option<rspack_plugin_runtime_chunk::RuntimeChunkOptions>,
}

impl OptimizationOptionsBuilder {
  pub fn remove_available_modules(&mut self, value: bool) -> &mut Self {
    self.remove_available_modules = Some(value);
    self
  }

  pub fn remove_empty_chunks(&mut self, value: bool) -> &mut Self {
    self.remove_empty_chunks = Some(value);
    self
  }

  pub fn merge_duplicate_chunks(&mut self, value: bool) -> &mut Self {
    self.merge_duplicate_chunks = Some(value);
    self
  }

  pub fn module_ids(&mut self, value: String) -> &mut Self {
    self.module_ids = Some(value);
    self
  }

  pub fn chunk_ids(&mut self, value: String) -> &mut Self {
    self.chunk_ids = Some(value);
    self
  }

  pub fn minimize(&mut self, value: bool) -> &mut Self {
    self.minimize = Some(value);
    self
  }

  pub fn minimizer(&mut self, value: Vec<BoxPlugin>) -> &mut Self {
    self.minimizer = Some(
      value
        .into_iter()
        .map(BuiltinPluginOptions::AnyMinimizerRspackPlugin)
        .collect(),
    );
    self
  }

  pub fn side_effects(&mut self, value: SideEffectOption) -> &mut Self {
    self.side_effects = Some(value);
    self
  }

  pub fn provided_exports(&mut self, value: bool) -> &mut Self {
    self.provided_exports = Some(value);
    self
  }

  pub fn used_exports(&mut self, value: UsedExportsOption) -> &mut Self {
    self.used_exports = Some(value);
    self
  }

  pub fn inner_graph(&mut self, value: bool) -> &mut Self {
    self.inner_graph = Some(value);
    self
  }

  pub fn mangle_exports(&mut self, value: MangleExportsOption) -> &mut Self {
    self.mangle_exports = Some(value);
    self
  }

  pub fn concatenate_modules(&mut self, value: bool) -> &mut Self {
    self.concatenate_modules = Some(value);
    self
  }

  pub fn real_content_hash(&mut self, value: bool) -> &mut Self {
    self.real_content_hash = Some(value);
    self
  }

  pub fn avoid_entry_iife(&mut self, value: bool) -> &mut Self {
    self.avoid_entry_iife = Some(value);
    self
  }

  pub fn node_env(&mut self, value: String) -> &mut Self {
    self.node_env = Some(value);
    self
  }

  pub fn emit_on_errors(&mut self, value: bool) -> &mut Self {
    self.emit_on_errors = Some(value);
    self
  }

  pub fn runtime_chunk(
    &mut self,
    value: rspack_plugin_runtime_chunk::RuntimeChunkOptions,
  ) -> &mut Self {
    self.runtime_chunk = Some(value);
    self
  }

  pub fn build(
    &mut self,
    builder_context: &mut BuilderContext,
    development: bool,
    production: bool,
    _css: bool,
  ) -> Optimization {
    let remove_available_modules = d!(self.remove_available_modules, false);
    let remove_empty_chunks = d!(self.remove_empty_chunks, true);
    if remove_empty_chunks {
      builder_context
        .plugins
        .push(BuiltinPluginOptions::RemoveEmptyChunksPlugin);
    }
    let real_content_hash = d!(self.real_content_hash, production);
    if real_content_hash {
      builder_context
        .plugins
        .push(BuiltinPluginOptions::RealContentHashPlugin);
    }
    let merge_duplicate_chunks = d!(self.merge_duplicate_chunks, true);
    if merge_duplicate_chunks {
      builder_context
        .plugins
        .push(BuiltinPluginOptions::MergeDuplicateChunksPlugin);
    }
    let module_ids = w!(self.module_ids, {
      if production {
        "deterministic".to_string()
      } else if development {
        "named".to_string()
      } else {
        "natural".to_string()
      }
    });

    match module_ids.as_str() {
      "deterministic" => {
        builder_context
          .plugins
          .push(BuiltinPluginOptions::DeterministicModuleIdsPlugin);
      }
      "named" => {
        builder_context
          .plugins
          .push(BuiltinPluginOptions::NamedModuleIdsPlugin);
      }
      "natural" => {
        builder_context
          .plugins
          .push(BuiltinPluginOptions::NaturalModuleIdsPlugin);
      }
      _ => {
        panic!("moduleIds: {module_ids} is not implemented");
      }
    }

    let chunk_ids = w!(self.chunk_ids, {
      if production {
        "deterministic".to_string()
      } else if development {
        "named".to_string()
      } else {
        "natural".to_string()
      }
    });

    match chunk_ids.as_str() {
      "deterministic" => {
        builder_context
          .plugins
          .push(BuiltinPluginOptions::DeterministicChunkIdsPlugin);
      }
      "named" => {
        builder_context
          .plugins
          .push(BuiltinPluginOptions::NamedChunkIdsPlugin);
      }
      "natural" => {
        builder_context
          .plugins
          .push(BuiltinPluginOptions::NaturalChunkIdsPlugin);
      }
      "size" => {
        builder_context
          .plugins
          .push(BuiltinPluginOptions::OccurrenceChunkIdsPlugin(
            rspack_ids::OccurrenceChunkIdsPluginOptions {
              prioritise_initial: true,
            },
          ));
      }
      "total-size" => {
        builder_context
          .plugins
          .push(BuiltinPluginOptions::OccurrenceChunkIdsPlugin(
            rspack_ids::OccurrenceChunkIdsPluginOptions {
              prioritise_initial: false,
            },
          ));
      }

      _ => {
        panic!("chunkIds: {chunk_ids} is not implemented");
      }
    }

    let side_effects = f!(self.side_effects.take(), || {
      if production {
        SideEffectOption::True
      } else {
        SideEffectOption::Flag
      }
    });
    if side_effects.is_enable() {
      builder_context
        .plugins
        .push(BuiltinPluginOptions::SideEffectsFlagPlugin);
    }

    let mangle_exports = f!(self.mangle_exports.take(), || {
      if production {
        MangleExportsOption::Deterministic
      } else {
        MangleExportsOption::False
      }
    });
    if mangle_exports.is_enable() {
      builder_context
        .plugins
        .push(BuiltinPluginOptions::MangleExportsPlugin(
          mangle_exports != MangleExportsOption::Size,
        ));
    }
    let provided_exports = d!(self.provided_exports, true);
    if provided_exports {
      builder_context
        .plugins
        .push(BuiltinPluginOptions::FlagDependencyExportsPlugin);
    }
    let used_exports = f!(self.used_exports.take(), || {
      if production {
        UsedExportsOption::True
      } else {
        UsedExportsOption::False
      }
    });
    if used_exports.is_enable() {
      builder_context
        .plugins
        .push(BuiltinPluginOptions::FlagDependencyUsagePlugin(
          used_exports.is_global(),
        ));
    }
    let inner_graph = d!(self.inner_graph, production);
    if !d!(self.emit_on_errors, !production) {
      builder_context
        .plugins
        .push(BuiltinPluginOptions::NoEmitOnErrorsPlugin);
    }

    if let Some(runtime_chunk) = self.runtime_chunk.take() {
      builder_context
        .plugins
        .push(BuiltinPluginOptions::RuntimeChunkPlugin(runtime_chunk));
    }

    let concatenate_modules = d!(self.concatenate_modules, production);
    if concatenate_modules {
      builder_context
        .plugins
        .push(BuiltinPluginOptions::ModuleConcatenationPlugin);
    }

    let avoid_entry_iife = d!(self.avoid_entry_iife, false);
    let minimize = d!(self.minimize, production);
    let minimizer = f!(self.minimizer.take(), || {
      if minimize {
        vec![
          BuiltinPluginOptions::AnyMinimizerRspackPlugin(Box::new(
            rspack_plugin_swc_js_minimizer::SwcJsMinimizerRspackPlugin::new(
              rspack_plugin_swc_js_minimizer::PluginOptions {
                test: None,
                include: None,
                exclude: None,
                extract_comments: None,
                minimizer_options: Default::default(),
              },
            ),
          )),
          // TODO: add lightning css
        ]
      } else {
        vec![]
      }
    });
    builder_context.plugins.extend(minimizer);

    if let Some(node_env) = self.node_env.take() {
      builder_context
        .plugins
        .push(BuiltinPluginOptions::DefinePlugin(
          [("process.env.NODE_ENV".to_string(), node_env.into())].into(),
        ));
    }

    Optimization {
      remove_available_modules,
      side_effects,
      provided_exports,
      used_exports,
      inner_graph,
      mangle_exports,
      concatenate_modules,
      avoid_entry_iife,
    }
  }
}

/// Builder used to build [`Experiments`]
#[derive(Debug, Default)]
pub struct ExperimentsBuilder {
  layers: Option<bool>,
  incremental: Option<IncrementalPasses>,
  top_level_await: Option<bool>,
  rspack_future: Option<RspackFuture>,
  cache: Option<ExperimentCacheOptions>,

  // Builder specific
  output_module: Option<bool>,
  future_defaults: Option<bool>,
  css: Option<bool>,
  async_web_assembly: Option<bool>,
  // TODO: lazy compilation
}

impl ExperimentsBuilder {
  pub fn layers(&mut self, layers: bool) -> &mut Self {
    self.layers = Some(layers);
    self
  }

  pub fn incremental(&mut self, incremental: IncrementalPasses) -> &mut Self {
    self.incremental = Some(incremental);
    self
  }

  pub fn top_level_await(&mut self, top_level_await: bool) -> &mut Self {
    self.top_level_await = Some(top_level_await);
    self
  }

  pub fn cache(&mut self, cache: ExperimentCacheOptions) -> &mut Self {
    self.cache = Some(cache);
    self
  }

  pub fn future_defaults(&mut self, future_defaults: bool) -> &mut Self {
    self.future_defaults = Some(future_defaults);
    self
  }

  pub fn css(&mut self, css: bool) -> &mut Self {
    self.css = Some(css);
    self
  }

  pub fn async_web_assembly(&mut self, async_web_assembly: bool) -> &mut Self {
    self.async_web_assembly = Some(async_web_assembly);
    self
  }

  pub fn build(
    &mut self,
    _builder_context: &mut BuilderContext,
    development: bool,
    production: bool,
  ) -> Experiments {
    let layers = d!(self.layers, false);
    let incremental = f!(self.incremental.take(), || {
      if !production {
        IncrementalPasses::MAKE | IncrementalPasses::EMIT_ASSETS
      } else {
        IncrementalPasses::empty()
      }
    });
    let top_level_await = d!(self.top_level_await, true);
    let cache = f!(self.cache.take(), || {
      if development {
        ExperimentCacheOptions::Memory
      } else {
        ExperimentCacheOptions::Disabled
      }
    });
    let rspack_future = d!(self.rspack_future.take(), RspackFuture {});

    // Builder specific
    let future_defaults = w!(self.future_defaults, false);
    w!(self.css, *future_defaults);
    w!(self.async_web_assembly, *future_defaults);
    w!(self.output_module, false);

    Experiments {
      layers,
      incremental,
      top_level_await,
      rspack_future,
      cache,
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn default() {
    let _ = CompilerOptions::builder().build(&mut Default::default());
  }

  #[test]
  fn builtin_plugin_order() {
    let mut context: BuilderContext = Default::default();
    let _ = CompilerOptions::builder()
      .mode(Mode::Production)
      .target(vec!["web".to_string()])
      .build(&mut context);
    context.plugins.sort_by_key(|p| p.tag());

    type BuiltinPluginOptionsTag = <BuiltinPluginOptions as EnumTag>::Tag;

    macro_rules! plugin_index {
      ($ident:ident) => {
        context
          .plugins
          .iter()
          .position(|p| p.tag() == BuiltinPluginOptionsTag::$ident)
          .expect("plugin should exist")
      };
    }

    let merge_duplicate_chunks_index = plugin_index!(MergeDuplicateChunksPlugin);
    let side_effects_flag_plugin_index = plugin_index!(SideEffectsFlagPlugin);
    let remove_empty_chunks_plugin_index = plugin_index!(RemoveEmptyChunksPlugin);
    let real_content_hash_plugin_index = plugin_index!(RealContentHashPlugin);

    assert!(merge_duplicate_chunks_index < side_effects_flag_plugin_index);
    assert!(remove_empty_chunks_plugin_index > merge_duplicate_chunks_index);
    assert!(real_content_hash_plugin_index > remove_empty_chunks_plugin_index);
  }
}
