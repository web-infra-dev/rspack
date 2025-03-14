//! The Rspack compiler builder.

mod builder_context;
mod devtool;
mod externals;
mod target;

pub use builder_context::BuilderContext;
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

use std::borrow::Cow;
use std::future::ready;
use std::sync::Arc;

use builder_context::BuiltinPluginOptions;
use devtool::DevtoolFlags;
use externals::ExternalsPresets;
use indexmap::IndexMap;
use rspack_core::{incremental::IncrementalPasses, ModuleType};
use rspack_core::{
  AssetParserDataUrl, AssetParserDataUrlOptions, AssetParserOptions, BoxPlugin, ByDependency,
  CacheOptions, ChunkLoading, ChunkLoadingType, CleanOptions, Compiler, CompilerOptions, Context,
  CrossOriginLoading, CssAutoGeneratorOptions, CssAutoParserOptions, CssExportsConvention,
  CssGeneratorOptions, CssModuleGeneratorOptions, CssModuleParserOptions, CssParserOptions,
  DynamicImportMode, EntryDescription, EntryOptions, EntryRuntime, Environment,
  ExperimentCacheOptions, Experiments, ExternalItem, ExternalType, Filename, FilenameTemplate,
  GeneratorOptions, GeneratorOptionsMap, JavascriptParserOptions, JavascriptParserOrder,
  JavascriptParserUrl, JsonGeneratorOptions, JsonParserOptions, LibraryName, LibraryNonUmdObject,
  LibraryOptions, LibraryType, MangleExportsOption, Mode, ModuleNoParseRules, ModuleOptions,
  ModuleRule, ModuleRuleEffect, NodeDirnameOption, NodeFilenameOption, NodeGlobalOption,
  NodeOption, Optimization, OutputOptions, ParseOption, ParserOptions, ParserOptionsMap, PathInfo,
  PublicPath, Resolve, RspackFuture, RuleSetCondition, RuleSetLogicalConditions, SideEffectOption,
  StatsOptions, TrustedTypes, UsedExportsOption, WasmLoading, WasmLoadingType,
};
use rspack_error::{
  miette::{self, Diagnostic},
  thiserror::{self, Error},
  Result,
};
use rspack_fs::{IntermediateFileSystem, ReadableFileSystem, WritableFileSystem};
use rspack_hash::{HashDigest, HashFunction, HashSalt};
use rspack_paths::{AssertUtf8, Utf8PathBuf};
use rspack_regex::RspackRegex;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use serde_json::json;
use target::{get_targets_properties, TargetProperties};

/// Error type for builder
#[derive(Debug, Clone, Error, Diagnostic)]
#[diagnostic()]
#[non_exhaustive]
pub enum BuilderError {
  /// Invalid option
  #[error("Invalid option '{0}': {1}")]
  Option(/* Accessor */ String, /* Error message */ String),
}

/// Builder trait
pub trait Builder {
  /// Target type
  type Item;

  /// Create a builder
  fn builder() -> Self::Item;
}

impl Builder for Compiler {
  type Item = CompilerBuilder;

  fn builder() -> Self::Item {
    CompilerBuilder::default()
  }
}

/// Builder used to build [`Compiler`].
///
/// [`Compiler`]: rspack_core::compiler::Compiler
#[derive(Default)]
pub struct CompilerBuilder {
  options_builder: CompilerOptionsBuilder,
  plugins: Vec<BoxPlugin>,
  input_filesystem: Option<Arc<dyn ReadableFileSystem>>,
  intermediate_filesystem: Option<Arc<dyn IntermediateFileSystem>>,
  output_filesystem: Option<Arc<dyn WritableFileSystem>>,
}

impl CompilerBuilder {
  /// Create a builder with options
  pub fn with_options<V>(options: V) -> Self
  where
    V: Into<CompilerOptionsBuilder>,
  {
    Self {
      options_builder: options.into(),
      plugins: vec![],
      input_filesystem: None,
      intermediate_filesystem: None,
      output_filesystem: None,
    }
  }
}

impl CompilerBuilder {
  /// Set the name of the configuration. Used when loading multiple configurations.
  ///
  /// See [`CompilerOptionsBuilder::name`] for more details.
  pub fn name(&mut self, name: String) -> &mut Self {
    self.options_builder.name(name);
    self
  }

  /// Set the target environment.
  ///
  /// See [`CompilerOptionsBuilder::target`] for more details.
  pub fn target(&mut self, targets: Targets) -> &mut Self {
    self.options_builder.target(targets);
    self
  }

  /// Set the entry point of the application.
  ///
  /// See [`CompilerOptionsBuilder::entry`] for more details.
  pub fn entry<K, V>(&mut self, entry_name: K, entry_description: V) -> &mut Self
  where
    K: Into<String>,
    V: Into<EntryDescription>,
  {
    self.options_builder.entry(entry_name, entry_description);
    self
  }

  /// Set the external libraries that should not be bundled.
  ///
  /// See [`CompilerOptionsBuilder::externals`] for more details.
  pub fn externals(&mut self, externals: ExternalItem) -> &mut Self {
    self.options_builder.externals(externals);
    self
  }

  /// Set the type of externals.
  ///
  /// See [`CompilerOptionsBuilder::externals_type`] for more details.
  pub fn externals_type(&mut self, externals_type: ExternalType) -> &mut Self {
    self.options_builder.externals_type(externals_type);
    self
  }

  /// Set the presets for external libraries.
  ///
  /// See [`CompilerOptionsBuilder::externals_presets`] for more details.
  pub fn externals_presets(&mut self, externals_presets: ExternalsPresets) -> &mut Self {
    self.options_builder.externals_presets(externals_presets);
    self
  }

  /// Set the context in which the compilation should occur.
  ///
  /// See [`CompilerOptionsBuilder::context`] for more details.
  pub fn context<V>(&mut self, context: V) -> &mut Self
  where
    V: Into<Context>,
  {
    self.options_builder.context(context);
    self
  }

  /// Set the cache options.
  ///
  /// See [`CompilerOptionsBuilder::cache`] for more details.
  pub fn cache(&mut self, cache: CacheOptions) -> &mut Self {
    self.options_builder.cache(cache);
    self
  }

  /// Set the source map configuration.
  ///
  /// See [`CompilerOptionsBuilder::devtool`] for more details.
  pub fn devtool(&mut self, devtool: Devtool) -> &mut Self {
    self.options_builder.devtool(devtool);
    self
  }

  /// Set the mode in which Rspack should operate.
  ///
  /// See [`CompilerOptionsBuilder::mode`] for more details.
  pub fn mode(&mut self, mode: Mode) -> &mut Self {
    self.options_builder.mode(mode);
    self
  }

  /// Set the resolve configuration.
  ///
  /// See [`CompilerOptionsBuilder::resolve`] for more details.
  pub fn resolve(&mut self, resolve: Resolve) -> &mut Self {
    self.options_builder.resolve(resolve);
    self
  }

  /// Set the resolve loader configuration.
  ///
  /// See [`CompilerOptionsBuilder::resolve_loader`] for more details.
  pub fn resolve_loader(&mut self, resolve_loader: Resolve) -> &mut Self {
    self.options_builder.resolve_loader(resolve_loader);
    self
  }

  /// Set whether to fail on the first error.
  ///
  /// See [`CompilerOptionsBuilder::bail`] for more details.
  pub fn bail(&mut self, bail: bool) -> &mut Self {
    self.options_builder.bail(bail);
    self
  }

  /// Set whether to enable profiling.
  ///
  /// See [`CompilerOptionsBuilder::profile`] for more details.
  pub fn profile(&mut self, profile: bool) -> &mut Self {
    self.options_builder.profile(profile);
    self
  }

  /// Set options for module configuration.
  ///
  /// Both are accepted:
  /// - [`ModuleOptionsBuilder`]
  /// - [`ModuleOptions`]
  ///
  /// # Examples
  ///
  /// ```rust
  /// use rspack::builder::{Builder as _, ModuleOptionsBuilder};
  /// use rspack_core::{Compiler, ModuleOptions};
  ///
  /// // Using builder without calling `build()`
  /// let compiler = Compiler::builder().module(ModuleOptionsBuilder::default().rules(vec![]));
  ///
  /// // `ModuleOptions::builder` equals to `ModuleOptionsBuilder::default()`
  /// let compiler = Compiler::builder().module(ModuleOptions::builder().rules(vec![]));
  ///
  /// // Directly passing `ModuleOptions`
  /// let compiler = Compiler::builder().module(ModuleOptions::default());
  /// ```
  ///
  /// See [`CompilerOptionsBuilder::module`] for more details.
  pub fn module<V>(&mut self, module: V) -> &mut Self
  where
    V: Into<ModuleOptionsBuilder>,
  {
    self.options_builder.module(module);
    self
  }

  /// Set options for output.
  ///
  /// Both are accepted:
  /// - [`OutputOptionsBuilder`]
  /// - [`OutputOptions`]
  ///
  /// # Examples
  ///
  /// ```rust
  /// use rspack::builder::{Builder as _, OutputOptionsBuilder};
  /// use rspack_core::{Compiler, OutputOptions};
  ///
  /// // Using builder without calling `build()`
  /// let compiler = Compiler::builder().output(OutputOptionsBuilder::default().path("/dist"));
  ///
  /// // `OutputOptions::builder` equals to `OutputOptionsBuilder::default()`
  /// let compiler = Compiler::builder().output(OutputOptions::builder().path("/dist"));
  ///
  /// // Or directly passing `OutputOptions`
  /// // let compiler = Compiler::builder().output(OutputOptions { ... });
  /// ```
  ///
  /// See [`CompilerOptionsBuilder::output`] for more details.
  pub fn output<V>(&mut self, output: V) -> &mut Self
  where
    V: Into<OutputOptionsBuilder>,
  {
    self.options_builder.output(output);
    self
  }

  /// Set options for optimization.
  ///
  /// Both are accepted:
  /// - [`OptimizationOptionsBuilder`]
  /// - [`Optimization`]
  ///
  /// # Examples
  ///
  /// ```rust
  /// use rspack::builder::{Builder as _, OptimizationOptionsBuilder};
  /// use rspack_core::{Compiler, Optimization};
  ///
  /// // Using builder without calling `build()`
  /// let compiler = Compiler::builder()
  ///   .optimization(OptimizationOptionsBuilder::default().remove_available_modules(true));
  ///
  /// // `Optimization::builder` equals to `OptimizationOptionsBuilder::default()`
  /// let compiler =
  ///   Compiler::builder().optimization(Optimization::builder().remove_available_modules(true));
  ///
  /// // Or directly passing `Optimization`
  /// // let compiler = Compiler::builder().optimization(Optimization { ... });
  /// ```
  ///
  /// See [`CompilerOptionsBuilder::optimization`] for more details.
  pub fn optimization<V>(&mut self, optimization: V) -> &mut Self
  where
    V: Into<OptimizationOptionsBuilder>,
  {
    self.options_builder.optimization(optimization);
    self
  }

  /// Set options for Node.js environment.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use rspack::builder::{Builder as _, NodeOptionBuilder};
  /// use rspack_core::{Compiler, NodeGlobalOption, NodeOption};
  ///
  /// // Using builder without calling `build()`
  /// let compiler =
  ///   Compiler::builder().node(NodeOptionBuilder::default().global(NodeGlobalOption::True));
  /// ```
  ///
  /// See [`CompilerOptionsBuilder::node`] for more details.
  pub fn node<V>(&mut self, node: V) -> &mut Self
  where
    V: Into<NodeOptionBuilder>,
  {
    self.options_builder.node(node);
    self
  }

  /// Set options for stats.
  ///
  /// See [`CompilerOptionsBuilder::stats`] for more details.
  pub fn stats(&mut self, stats: StatsOptions) -> &mut Self {
    self.options_builder.stats(stats);
    self
  }

  /// Set the value of `require.amd` or `define.amd`.
  ///
  /// See [`CompilerOptionsBuilder::amd`] for more details.
  pub fn amd(&mut self, amd: String) -> &mut Self {
    self.options_builder.amd(amd);
    self
  }

  /// Set options for experiments.
  ///
  /// Both are accepted:
  /// - [`ExperimentsBuilder`]
  /// - [`Experiments`]
  ///
  /// # Examples
  ///
  /// ```rust
  /// use rspack::builder::{Builder as _, ExperimentsBuilder};
  /// use rspack_core::incremental::IncrementalPasses;
  /// use rspack_core::{Compiler, Experiments};
  ///
  /// // Using builder without calling `build()`
  /// let compiler = Compiler::builder()
  ///   .experiments(ExperimentsBuilder::default().incremental(IncrementalPasses::empty()));
  ///
  /// // `Experiments::builder` equals to `ExperimentsBuilder::default()`
  /// let compiler =
  ///   Compiler::builder().experiments(Experiments::builder().incremental(IncrementalPasses::empty()));
  ///
  /// // Or directly passing `Experiments`
  /// // let compiler = Compiler::builder().experiments(Experiments { ... });
  /// ```
  ///
  /// See [`CompilerOptionsBuilder::experiments`] for more details.
  pub fn experiments<V>(&mut self, experiments: V) -> &mut Self
  where
    V: Into<ExperimentsBuilder>,
  {
    self.options_builder.experiments(experiments);
    self
  }

  /// Add a plugin to the compiler.
  pub fn plugin(&mut self, plugin: BoxPlugin) -> &mut Self {
    self.plugins.push(plugin);
    self
  }

  /// Add plugins to the compiler.
  pub fn plugins(&mut self, plugins: impl IntoIterator<Item = BoxPlugin>) -> &mut Self {
    self.plugins.extend(plugins);
    self
  }

  /// Set the input filesystem.
  pub fn input_filesystem(&mut self, input_filesystem: Arc<dyn ReadableFileSystem>) -> &mut Self {
    self.input_filesystem = Some(input_filesystem);
    self
  }

  /// Set the intermediate filesystem.
  pub fn intermediate_filesystem(
    &mut self,
    intermediate_filesystem: Arc<dyn IntermediateFileSystem>,
  ) -> &mut Self {
    self.intermediate_filesystem = Some(intermediate_filesystem);
    self
  }

  /// Set the output filesystem.
  pub fn output_filesystem(&mut self, output_filesystem: Arc<dyn WritableFileSystem>) -> &mut Self {
    self.output_filesystem = Some(output_filesystem);
    self
  }

  /// Build [`Compiler`] from options and plugins.
  pub fn build(&mut self) -> Result<Compiler> {
    let mut builder_context = BuilderContext::default();
    let compiler_options = self.options_builder.build(&mut builder_context)?;
    let mut plugins = builder_context.take_plugins(&compiler_options);
    plugins.append(&mut self.plugins);

    let input_filesystem = self.input_filesystem.take();
    let intermediate_filesystem = self.intermediate_filesystem.take();
    let output_filesystem = self.output_filesystem.take();

    Ok(Compiler::new(
      String::new(),
      compiler_options,
      plugins,
      vec![],
      output_filesystem,
      intermediate_filesystem,
      input_filesystem,
      None,
      None,
    ))
  }
}

#[cfg(feature = "loader_lightningcss")]
impl CompilerBuilder {
  /// Enable support for builtin:lightningcss-loader.
  pub fn enable_loader_lightningcss(&mut self) -> &mut Self {
    self.plugin(Box::new(
      rspack_loader_lightningcss::LightningcssLoaderPlugin::new(),
    ))
  }
}

#[cfg(feature = "loader_swc")]
impl CompilerBuilder {
  /// Enable support for builtin:swc-loader.
  pub fn enable_loader_swc(&mut self) -> &mut Self {
    self.plugin(Box::new(rspack_loader_swc::SwcLoaderPlugin::new()))
  }
}

#[cfg(feature = "loader_react_refresh")]
impl CompilerBuilder {
  /// Enable support for builtin:react-refresh-loader.
  pub fn enable_loader_react_refresh(&mut self) -> &mut Self {
    self.plugin(Box::new(
      rspack_loader_react_refresh::ReactRefreshLoaderPlugin::new(),
    ))
  }
}

#[cfg(feature = "loader_preact_refresh")]
impl CompilerBuilder {
  /// Enable support for builtin:preact-refresh-loader.
  pub fn enable_loader_preact_refresh(&mut self) -> &mut Self {
    self.plugin(Box::new(
      rspack_loader_preact_refresh::PreactRefreshLoaderPlugin::new(),
    ))
  }
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

impl Builder for Option<NodeOption> {
  type Item = NodeOptionBuilder;

  fn builder() -> Self::Item {
    NodeOptionBuilder::default()
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

/// Builder used to build [`CompilerOptions`]
#[derive(Debug, Default)]
pub struct CompilerOptionsBuilder {
  /// Name of the configuration. Used when loading multiple configurations.
  name: Option<String>,
  /// The environment in which the code should run.
  target: Option<Targets>,
  /// The entry point of the application.
  entry: IndexMap<String, EntryDescription>,
  /// External libraries that should not be bundled.
  externals: Option<Vec<ExternalItem>>,
  /// The type of externals.
  externals_type: Option<ExternalType>,
  /// Presets for external libraries.
  externals_presets: Option<ExternalsPresets>,
  /// The context in which the compilation should occur.
  context: Option<Context>,
  /// Options for caching.
  cache: Option<CacheOptions>,
  /// The mode in which Rspack should operate.
  mode: Option<Mode>,
  /// The type of externals.
  resolve: Option<Resolve>,
  /// The type of externals.
  resolve_loader: Option<Resolve>,
  /// The type of externals.
  devtool: Option<Devtool>,
  /// The type of externals.
  profile: Option<bool>,
  /// Whether to fail on the first error.
  bail: Option<bool>,
  /// Performance optimization options.
  experiments: Option<ExperimentsBuilder>,
  /// Options for module configuration.
  module: Option<ModuleOptionsBuilder>,
  /// Options for stats.
  stats: Option<StatsOptions>,
  /// Options for output.
  output: Option<OutputOptionsBuilder>,
  /// Optimization options.
  optimization: Option<OptimizationOptionsBuilder>,
  /// Options for Node.js environment.
  node: Option<NodeOptionBuilder>,
  /// The value of `require.amd` or `define.amd`.
  amd: Option<String>,
}

impl From<&mut CompilerOptionsBuilder> for CompilerOptionsBuilder {
  fn from(value: &mut CompilerOptionsBuilder) -> Self {
    CompilerOptionsBuilder {
      name: value.name.take(),
      target: value.target.take(),
      entry: value.entry.clone(),
      externals: value.externals.take(),
      externals_type: value.externals_type.take(),
      externals_presets: value.externals_presets.take(),
      context: value.context.take(),
      cache: value.cache.take(),
      mode: value.mode.take(),
      resolve: value.resolve.take(),
      resolve_loader: value.resolve_loader.take(),
      devtool: value.devtool.take(),
      profile: value.profile.take(),
      bail: value.bail.take(),
      experiments: value.experiments.take(),
      module: value.module.take(),
      output: value.output.take(),
      stats: value.stats.take(),
      optimization: value.optimization.take(),
      node: value.node.take(),
      amd: value.amd.take(),
    }
  }
}

impl CompilerOptionsBuilder {
  /// Set the name of the configuration. Used when loading multiple configurations.
  pub fn name(&mut self, name: String) -> &mut Self {
    self.name = Some(name);
    self
  }

  /// Set the environment in which the code should run.
  pub fn target(&mut self, targets: Targets) -> &mut Self {
    self.target = Some(targets);
    self
  }

  /// Add an entry point to the configuration.
  pub fn entry<K, V>(&mut self, entry_name: K, entry_description: V) -> &mut Self
  where
    K: Into<String>,
    V: Into<EntryDescription>,
  {
    self
      .entry
      .insert(entry_name.into(), entry_description.into());
    self
  }

  /// Set external libraries that should not be bundled.
  pub fn externals(&mut self, externals: ExternalItem) -> &mut Self {
    match &mut self.externals {
      Some(e) => e.push(externals),
      None => self.externals = Some(vec![externals]),
    }
    self
  }

  /// Set the type of externals.
  pub fn externals_type(&mut self, externals_type: ExternalType) -> &mut Self {
    self.externals_type = Some(externals_type);
    self
  }

  /// Set presets for external libraries.
  pub fn externals_presets(&mut self, externals_presets: ExternalsPresets) -> &mut Self {
    self.externals_presets = Some(externals_presets);
    self
  }

  /// Set the context in which the compilation should occur.
  pub fn context<V>(&mut self, context: V) -> &mut Self
  where
    V: Into<Context>,
  {
    self.context = Some(context.into());
    self
  }

  /// Set options for caching.
  pub fn cache(&mut self, cache: CacheOptions) -> &mut Self {
    self.cache = Some(cache);
    self
  }

  /// Set the source map configuration.
  pub fn devtool(&mut self, devtool: Devtool) -> &mut Self {
    self.devtool = Some(devtool);
    self
  }

  /// Set the mode in which Rspack should operate.
  pub fn mode(&mut self, mode: Mode) -> &mut Self {
    self.mode = Some(mode);
    self
  }

  /// Set the resolve configuration.
  pub fn resolve(&mut self, resolve: Resolve) -> &mut Self {
    self.resolve = Some(resolve);
    self
  }

  /// Set the resolve loader configuration.
  pub fn resolve_loader(&mut self, resolve_loader: Resolve) -> &mut Self {
    self.resolve_loader = Some(resolve_loader);
    self
  }

  /// Set whether to fail on the first error.
  pub fn bail(&mut self, bail: bool) -> &mut Self {
    self.bail = Some(bail);
    self
  }

  /// Set whether to enable profiling.
  pub fn profile(&mut self, profile: bool) -> &mut Self {
    self.profile = Some(profile);
    self
  }

  /// Set options for module configuration.
  ///
  /// Both are accepted:
  /// - [`ModuleOptionsBuilder`]
  /// - [`ModuleOptions`]
  ///
  /// # Examples
  ///
  /// ```rust
  /// use rspack::builder::{Builder as _, ModuleOptionsBuilder};
  /// use rspack_core::{Compiler, ModuleOptions};
  ///
  /// // Using builder without calling `build()`
  /// let compiler = Compiler::builder().module(ModuleOptionsBuilder::default().rules(vec![]));
  ///
  /// // `ModuleOptions::builder` equals to `ModuleOptionsBuilder::default()`
  /// let compiler = Compiler::builder().module(ModuleOptions::builder().rules(vec![]));
  ///
  /// // Directly passing `ModuleOptions`
  /// let compiler = Compiler::builder().module(ModuleOptions::default());
  /// ```
  pub fn module<V>(&mut self, module: V) -> &mut Self
  where
    V: Into<ModuleOptionsBuilder>,
  {
    self.module = Some(module.into());
    self
  }

  /// Set options for stats.
  pub fn stats(&mut self, stats: StatsOptions) -> &mut Self {
    self.stats = Some(stats);
    self
  }

  /// Set options for output.
  ///
  /// Both are accepted:
  /// - [`OutputOptionsBuilder`]
  /// - [`OutputOptions`]
  ///
  /// # Examples
  ///
  /// ```rust
  /// use rspack::builder::{Builder as _, OutputOptionsBuilder};
  /// use rspack_core::{Compiler, OutputOptions};
  ///
  /// // Using builder without calling `build()`
  /// let compiler = Compiler::builder().output(OutputOptionsBuilder::default().path("/dist"));
  ///
  /// // `OutputOptions::builder` equals to `OutputOptionsBuilder::default()`
  /// let compiler = Compiler::builder().output(OutputOptions::builder().path("/dist"));
  ///
  /// // Or directly passing `OutputOptions`
  /// // let compiler = Compiler::builder().output(OutputOptions { ... });
  /// ```
  pub fn output<V>(&mut self, output: V) -> &mut Self
  where
    V: Into<OutputOptionsBuilder>,
  {
    self.output = Some(output.into());
    self
  }

  /// Set options for optimization.  
  ///
  /// Both are accepted:
  /// - [`OptimizationOptionsBuilder`]
  /// - [`Optimization`]
  ///
  /// # Examples
  ///
  /// ```rust
  /// use rspack::builder::{Builder as _, OptimizationOptionsBuilder};
  /// use rspack_core::{Compiler, Optimization};
  ///
  /// // Using builder without calling `build()`
  /// let compiler = Compiler::builder()
  ///   .optimization(OptimizationOptionsBuilder::default().remove_available_modules(true));
  ///
  /// // `Optimization::builder` equals to `OptimizationOptionsBuilder::default()`
  /// let compiler =
  ///   Compiler::builder().optimization(Optimization::builder().remove_available_modules(true));
  ///
  /// // Or directly passing `Optimization`
  /// // let compiler = Compiler::builder().optimization(Optimization { ... });
  /// ```
  pub fn optimization<V>(&mut self, optimization: V) -> &mut Self
  where
    V: Into<OptimizationOptionsBuilder>,
  {
    self.optimization = Some(optimization.into());
    self
  }

  /// Set options for Node.js environment.
  ///
  ///
  /// # Examples
  ///
  /// ```rust
  /// use rspack::builder::{Builder as _, NodeOptionBuilder};
  /// use rspack_core::{Compiler, NodeGlobalOption, NodeOption};
  ///
  /// // Using builder without calling `build()`
  /// let compiler =
  ///   Compiler::builder().node(NodeOptionBuilder::default().global(NodeGlobalOption::True));
  /// ```
  pub fn node<V>(&mut self, node: V) -> &mut Self
  where
    V: Into<NodeOptionBuilder>,
  {
    self.node = Some(node.into());
    self
  }

  /// Set the value of `require.amd` or `define.amd`.
  pub fn amd(&mut self, amd: String) -> &mut Self {
    self.amd = Some(amd);
    self
  }

  /// Set options for experiments.
  ///
  /// Both are accepted:
  /// - [`ExperimentsBuilder`]
  /// - [`Experiments`]
  ///
  /// # Examples
  ///
  /// ```rust
  /// use rspack::builder::{Builder as _, ExperimentsBuilder};
  /// use rspack_core::incremental::IncrementalPasses;
  /// use rspack_core::{Compiler, Experiments};
  ///
  /// // Using builder without calling `build()`
  /// let compiler = Compiler::builder()
  ///   .experiments(ExperimentsBuilder::default().incremental(IncrementalPasses::empty()));
  ///
  /// // `Experiments::builder` equals to `ExperimentsBuilder::default()`
  /// let compiler =
  ///   Compiler::builder().experiments(Experiments::builder().incremental(IncrementalPasses::empty()));
  ///
  /// // Or directly passing `Experiments`
  /// // let compiler = Compiler::builder().experiments(Experiments { ... });
  /// ```
  pub fn experiments<V>(&mut self, experiments: V) -> &mut Self
  where
    V: Into<ExperimentsBuilder>,
  {
    self.experiments = Some(experiments.into());
    self
  }

  /// Build the options for the compiler, return [`CompilerOptions`].
  ///
  /// The returned [`CompilerOptions`] is ready to use for creating compiler.
  /// Plugins created by this method will be added to the [`BuilderContext`], which will be used to create the compiler.
  ///
  /// To create the compiler, you can use [`Compiler::builder`].
  ///
  /// [`BuilderContext`]: crate::builder::BuilderContext
  /// [`CompilerOptions`]: rspack_core::options::CompilerOptions
  pub fn build(&mut self, builder_context: &mut BuilderContext) -> Result<CompilerOptions> {
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

    if self.entry.is_empty() {
      self
        .entry
        .insert("main".to_string(), EntryDescription::default());
    }
    self.entry.iter_mut().for_each(|(_, entry)| {
      entry.import.get_or_insert(vec!["./src".to_string()]);
    });

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
        CacheOptions::Memory {
          max_generations: None,
        }
      } else {
        CacheOptions::Disabled
      }
    });

    // apply experiments defaults
    let mut experiments_builder = f!(self.experiments.take(), Experiments::builder);
    let mut experiments = experiments_builder.build(builder_context, development, production)?;
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
    )?;

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
    )?;

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
        debug_ids: false,
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

    // apply node defaults
    let node = f!(self.node.take(), <Option<NodeOption>>::builder)
      .build(&target_properties, output_module)?;

    // apply optimization defaults
    let optimization = f!(self.optimization.take(), Optimization::builder).build(
      builder_context,
      development,
      production,
      css,
    )?;

    // apply resolve defaults
    let resolve = {
      let resolve_defaults = get_resolve_defaults(&context, mode, &target_properties, css);
      if let Some(resolve) = self.resolve.take() {
        resolve_defaults.merge(resolve)
      } else {
        resolve_defaults
      }
    };

    // apply resolve loader defaults
    let resolve_loader = {
      let resolve_loader_defaults = get_resolve_loader_defaults();
      if let Some(resolve_loader) = self.resolve_loader.take() {
        resolve_loader_defaults.merge(resolve_loader)
      } else {
        resolve_loader_defaults
      }
    };

    // apply entry plugin
    std::mem::take(&mut self.entry)
      .into_iter()
      .for_each(|(name, desc)| {
        let entry_options = EntryOptions {
          name: Some(name),
          runtime: desc.runtime.map(EntryRuntime::String),
          chunk_loading: desc.chunk_loading,
          async_chunks: desc.async_chunks,
          public_path: desc.public_path,
          base_uri: desc.base_uri,
          filename: desc.filename,
          library: None,
          depend_on: desc.depend_on,
          layer: None,
        };
        // SAFETY: `desc.import` is not `None` as entry has been normalized above.
        expect!(desc.import).into_iter().for_each(|import| {
          builder_context
            .plugins
            .push(BuiltinPluginOptions::EntryPlugin((
              import,
              entry_options.clone(),
            )));
        });
      });

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
    builder_context
      .plugins
      .push(BuiltinPluginOptions::EnsureChunkConditionsPlugin);
    builder_context
      .plugins
      .push(BuiltinPluginOptions::WorkerPlugin);

    // TODO: stats plugins
    let stats = d!(self.stats.take(), StatsOptions { colors: true });

    let amd = self.amd.take();

    Ok(CompilerOptions {
      name,
      context,
      output,
      mode,
      resolve,
      resolve_loader,
      module,
      stats,
      cache,
      experiments,
      node,
      optimization,
      profile,
      amd,
      bail,
      __references: Default::default(),
    })
  }
}

fn get_resolve_defaults(
  context: &Context,
  mode: Mode,
  target_properties: &TargetProperties,
  css: bool,
) -> Resolve {
  let mut conditions = vec!["webpack".to_string()];

  // Add mode condition
  conditions.push(match mode {
    Mode::Development => "development".to_string(),
    _ => "production".to_string(),
  });

  // Add target conditions
  if target_properties.webworker() {
    conditions.push("worker".to_string());
  }
  if target_properties.node() {
    conditions.push("node".to_string());
  }
  if target_properties.web() {
    conditions.push("browser".to_string());
  }
  if target_properties.electron() {
    conditions.push("electron".to_string());
  }
  if target_properties.nwjs() {
    conditions.push("nwjs".to_string());
  }

  let js_extensions = vec![".js".to_string(), ".json".to_string(), ".wasm".to_string()];

  let browser_field = target_properties.web()
    && (!target_properties.node()
      || (target_properties.electron() && target_properties.electron_renderer()));

  let alias_fields: Vec<Vec<String>> = (if browser_field {
    vec!["browser".to_string()]
  } else {
    vec![]
  })
  .into_iter()
  .map(|field| vec![field])
  .collect();

  let main_fields = if browser_field {
    vec![
      "browser".to_string(),
      "module".to_string(),
      "...".to_string(),
    ]
  } else {
    vec!["module".to_string(), "...".to_string()]
  };

  let cjs_deps = || Resolve {
    alias_fields: Some(alias_fields.clone()),
    main_fields: Some(main_fields.clone()),
    condition_names: Some(vec![
      "require".to_string(),
      "module".to_string(),
      "...".to_string(),
    ]),
    extensions: Some(js_extensions.clone()),
    ..Default::default()
  };

  let esm_deps = || Resolve {
    alias_fields: Some(alias_fields.clone()),
    main_fields: Some(main_fields.clone()),
    condition_names: Some(vec![
      "import".to_string(),
      "module".to_string(),
      "...".to_string(),
    ]),
    extensions: Some(js_extensions.clone()),
    ..Default::default()
  };

  let mut by_dependency: Vec<(Cow<'static, str>, Resolve)> = vec![
    ("wasm".into(), esm_deps()),
    ("esm".into(), esm_deps()),
    (
      "url".into(),
      Resolve {
        prefer_relative: Some(true),
        ..Default::default()
      },
    ),
    (
      "worker".into(),
      Resolve {
        prefer_relative: Some(true),
        ..esm_deps()
      },
    ),
    ("commonjs".into(), cjs_deps()),
    ("amd".into(), cjs_deps()),
    ("unknown".into(), cjs_deps()),
  ];

  // Add CSS dependencies if enabled
  if css {
    let mut style_conditions = vec!["webpack".to_string()];
    style_conditions.push(match mode {
      Mode::Development => "development".to_string(),
      _ => "production".to_string(),
    });
    style_conditions.push("style".to_string());

    by_dependency.push((
      "css-import".into(),
      Resolve {
        main_files: Some(vec![]),
        main_fields: Some(vec!["style".to_string(), "...".to_string()]),
        condition_names: Some(style_conditions),
        extensions: Some(vec![".css".to_string()]),
        prefer_relative: Some(true),
        ..Default::default()
      },
    ));
  }

  Resolve {
    modules: Some(vec!["node_modules".to_string()]),
    condition_names: Some(conditions),
    main_files: Some(vec!["index".to_string()]),
    extensions: Some(vec![]),
    alias_fields: Some(vec![]),
    exports_fields: Some(vec![vec!["exports".to_string()]]),
    roots: Some(vec![context.to_string()]),
    main_fields: Some(vec!["main".to_string()]),
    imports_fields: Some(vec![vec!["imports".to_string()]]),
    by_dependency: Some(ByDependency::from_iter(by_dependency)),
    ..Default::default()
  }
}

fn get_resolve_loader_defaults() -> Resolve {
  Resolve {
    condition_names: Some(vec![
      "loader".to_string(),
      "require".to_string(),
      "node".to_string(),
    ]),
    extensions: Some(vec![".js".to_string()]),
    main_fields: Some(vec!["loader".to_string(), "main".to_string()]),
    main_files: Some(vec!["index".to_string()]),
    exports_fields: Some(vec![vec!["exports".to_string()]]),
    ..Default::default()
  }
}

/// Builder used to build [`NodeOption`].
///
/// [`NodeOption`]: rspack_core::options::NodeOption
#[derive(Debug)]
pub enum NodeOptionBuilder {
  /// Set options for Node.js environment.
  True {
    /// Set the `__dirname` for Node.js environment.
    dirname: Option<NodeDirnameOption>,
    /// Set the `global` for Node.js environment.
    global: Option<NodeGlobalOption>,
    /// Set the `__filename` for Node.js environment.
    filename: Option<NodeFilenameOption>,
  },
  /// Disable Node.js environment.
  False,
}

impl From<Option<NodeOption>> for NodeOptionBuilder {
  fn from(value: Option<NodeOption>) -> Self {
    match value {
      Some(node_option) => NodeOptionBuilder::True {
        dirname: Some(node_option.dirname),
        global: Some(node_option.global),
        filename: Some(node_option.filename),
      },
      None => NodeOptionBuilder::False,
    }
  }
}

impl From<&mut NodeOptionBuilder> for NodeOptionBuilder {
  fn from(value: &mut NodeOptionBuilder) -> Self {
    match value {
      NodeOptionBuilder::True {
        dirname,
        global,
        filename,
      } => NodeOptionBuilder::True {
        dirname: dirname.take(),
        global: global.take(),
        filename: filename.take(),
      },
      NodeOptionBuilder::False => NodeOptionBuilder::False,
    }
  }
}

impl Default for NodeOptionBuilder {
  fn default() -> Self {
    NodeOptionBuilder::True {
      dirname: None,
      global: None,
      filename: None,
    }
  }
}

impl NodeOptionBuilder {
  /// Disable Node.js environment.
  pub fn disabled(&mut self) -> &mut Self {
    *self = NodeOptionBuilder::False;
    self
  }

  /// Set the `__dirname` for Node.js environment.
  pub fn dirname(&mut self, dirname: NodeDirnameOption) -> &mut Self {
    match self {
      NodeOptionBuilder::True { dirname: d, .. } => {
        *d = Some(dirname);
      }
      NodeOptionBuilder::False => {
        *self = NodeOptionBuilder::True {
          dirname: Some(dirname),
          global: None,
          filename: None,
        }
      }
    }
    self
  }

  /// Set the `global` for Node.js environment.
  pub fn global(&mut self, global: NodeGlobalOption) -> &mut Self {
    match self {
      NodeOptionBuilder::True { global: g, .. } => {
        *g = Some(global);
      }
      NodeOptionBuilder::False => {
        *self = NodeOptionBuilder::True {
          dirname: None,
          global: Some(global),
          filename: None,
        }
      }
    }
    self
  }

  /// Set the `__filename` for Node.js environment.
  pub fn filename(&mut self, filename: NodeFilenameOption) -> &mut Self {
    match self {
      NodeOptionBuilder::True { filename: f, .. } => {
        *f = Some(filename);
      }
      NodeOptionBuilder::False => {
        *self = NodeOptionBuilder::True {
          dirname: None,
          global: None,
          filename: Some(filename),
        }
      }
    }
    self
  }

  /// Build [`NodeOption`] from options.
  ///
  /// [`NodeOption`]: rspack_core::options::NodeOption
  fn build(
    &mut self,
    target_properties: &TargetProperties,
    output_module: bool,
  ) -> Result<Option<NodeOption>> {
    match self {
      NodeOptionBuilder::True {
        dirname,
        global,
        filename,
      } => {
        if global.is_none() {
          if target_properties.global() {
            *global = Some(NodeGlobalOption::False);
          } else {
            *global = Some(NodeGlobalOption::Warn);
          }
        }
        if dirname.is_none() {
          if target_properties.node() && output_module {
            *dirname = Some(NodeDirnameOption::NodeModule);
          } else if target_properties.node() {
            *dirname = Some(NodeDirnameOption::EvalOnly);
          } else {
            *dirname = Some(NodeDirnameOption::WarnMock);
          }
        }
        if filename.is_none() {
          if target_properties.node() && output_module {
            *filename = Some(NodeFilenameOption::NodeModule);
          } else if target_properties.node() {
            *filename = Some(NodeFilenameOption::EvalOnly);
          } else {
            *filename = Some(NodeFilenameOption::WarnMock);
          }
        }
        Ok(Some(NodeOption {
          dirname: expect!(dirname.take()),
          global: expect!(global.take()),
          filename: expect!(filename.take()),
        }))
      }
      NodeOptionBuilder::False => Ok(None),
    }
  }
}

/// Builder used to build [`ModuleOptions`].
///
/// [`ModuleOptions`]: rspack_core::options::ModuleOptions
#[derive(Debug, Default)]
pub struct ModuleOptionsBuilder {
  /// An array of rules that match the module's requests when it is created.
  rules: Vec<ModuleRule>,
  /// Configure all parsers' options in one place with module.parser.
  parser: Option<ParserOptionsMap>,
  /// Configure all generators' options in one place with module.generator.
  generator: Option<GeneratorOptionsMap>,
  /// Keep module mechanism of the matched modules as-is, such as module.exports, require, import.
  no_parse: Option<ModuleNoParseRules>,
}

impl From<ModuleOptions> for ModuleOptionsBuilder {
  fn from(value: ModuleOptions) -> Self {
    ModuleOptionsBuilder {
      rules: value.rules,
      parser: value.parser,
      generator: value.generator,
      no_parse: value.no_parse,
    }
  }
}

impl From<&mut ModuleOptionsBuilder> for ModuleOptionsBuilder {
  fn from(value: &mut ModuleOptionsBuilder) -> Self {
    ModuleOptionsBuilder {
      rules: std::mem::take(&mut value.rules),
      parser: value.parser.take(),
      generator: value.generator.take(),
      no_parse: value.no_parse.take(),
    }
  }
}

impl ModuleOptionsBuilder {
  /// Add a rule to the module.
  pub fn rule(&mut self, rule: ModuleRule) -> &mut Self {
    self.rules.push(rule);
    self
  }

  /// Add multiple rules to the module.
  pub fn rules(&mut self, mut rules: Vec<ModuleRule>) -> &mut Self {
    self.rules.append(&mut rules);
    self
  }

  /// Set the parser options for the module.
  pub fn parser(&mut self, parser: ParserOptionsMap) -> &mut Self {
    match &mut self.parser {
      Some(p) => p.extend(parser.clone()),
      None => self.parser = Some(parser),
    }
    self
  }

  /// Set the generator options for the module.
  pub fn generator(&mut self, generator: GeneratorOptionsMap) -> &mut Self {
    match &mut self.generator {
      Some(g) => g.extend(generator.clone()),
      None => self.generator = Some(generator),
    }
    self
  }

  /// Set the no_parse options for the module.
  pub fn no_parse(&mut self, no_parse: ModuleNoParseRules) -> &mut Self {
    self.no_parse = Some(no_parse);
    self
  }

  /// Build [`ModuleOptions`] from options.
  ///
  /// Normally, you don't need to call this function, it's used internally by [`CompilerBuilder::build`].
  ///
  /// [`ModuleOptions`]: rspack_core::options::ModuleOptions
  fn build(
    &mut self,
    _builder_context: &mut BuilderContext,
    async_web_assembly: bool,
    css: bool,
    target_properties: &TargetProperties,
    mode: &Mode,
  ) -> Result<ModuleOptions> {
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

    let generator = self.generator.get_or_insert(GeneratorOptionsMap::default());
    if !generator.contains_key("json") {
      generator.insert(
        "json".to_string(),
        GeneratorOptions::Json(JsonGeneratorOptions {
          json_parse: Some(true),
        }),
      );
    }

    if css {
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

    Ok(ModuleOptions {
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
    })
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
      test: Some(RuleSetCondition::Func(Box::new(|ctx| {
        Box::pin(ready(Ok(
          ctx
            .as_str()
            .map(|data| data.ends_with(".json"))
            .unwrap_or_default(),
        )))
      }))),
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
      test: Some(RuleSetCondition::Func(Box::new(|ctx| {
        Box::pin(ready(Ok(
          ctx
            .as_str()
            .map(|data| data.ends_with(".mjs"))
            .unwrap_or_default(),
        )))
      }))),
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
      test: Some(RuleSetCondition::Func(Box::new(|ctx| {
        Box::pin(ready(Ok(
          ctx
            .as_str()
            .map(|data| data.ends_with(".js"))
            .unwrap_or_default(),
        )))
      }))),
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
      test: Some(RuleSetCondition::Func(Box::new(|ctx| {
        Box::pin(ready(Ok(
          ctx
            .as_str()
            .map(|data| data.ends_with(".cjs"))
            .unwrap_or_default(),
        )))
      }))),
      effect: ModuleRuleEffect {
        r#type: Some(ModuleType::JsDynamic),
        ..Default::default()
      },
      ..Default::default()
    },
    // .js with type:commonjs
    ModuleRule {
      test: Some(RuleSetCondition::Func(Box::new(|ctx| {
        Box::pin(ready(Ok(
          ctx
            .as_str()
            .map(|data| data.ends_with(".js"))
            .unwrap_or_default(),
        )))
      }))),
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
        test: Some(RuleSetCondition::Func(Box::new(|ctx| {
          Box::pin(ready(Ok(
            ctx
              .as_str()
              .map(|data| data.ends_with(".wasm"))
              .unwrap_or_default(),
          )))
        }))),
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
        test: Some(RuleSetCondition::Func(Box::new(|ctx| {
          Box::pin(ready(Ok(
            ctx
              .as_str()
              .map(|data| data.ends_with(".css"))
              .unwrap_or_default(),
          )))
        }))),
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
          scheme: Some(RuleSetCondition::String("data".into()).into()),
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

/// Builder used to build [`OutputOptions`].
///
/// [`OutputOptions`]: rspack_core::options::OutputOptions
#[derive(Debug, Default)]
pub struct OutputOptionsBuilder {
  /// Set the output path.
  path: Option<Utf8PathBuf>,
  /// Set the pathinfo option.
  pathinfo: Option<PathInfo>,
  /// Set the clean option.
  clean: Option<CleanOptions>,
  /// Set the public path.
  public_path: Option<PublicPath>,
  /// Set the asset module filename.
  asset_module_filename: Option<Filename>,
  /// Set the wasm loading.
  wasm_loading: Option<WasmLoading>,
  /// Set the wasm module filename.
  webassembly_module_filename: Option<FilenameTemplate>,
  /// Set the unique name.
  unique_name: Option<String>,
  /// Set the chunk loading.
  chunk_loading: Option<ChunkLoading>,
  /// Set the chunk loading global.
  chunk_loading_global: Option<String>,
  /// Set the chunk load timeout.
  chunk_load_timeout: Option<u32>,
  /// Set the chunk format.
  chunk_format: Option<String>,
  /// Set the charset.
  charset: Option<bool>,
  /// Set the filename.
  filename: Option<Filename>,
  /// Set the chunk filename.
  chunk_filename: Option<Filename>,
  /// Set the cross origin loading.
  cross_origin_loading: Option<CrossOriginLoading>,
  /// Set the css filename.
  css_filename: Option<Filename>,
  /// Set the css chunk filename.
  css_chunk_filename: Option<Filename>,
  /// Set the hot update main filename.
  hot_update_main_filename: Option<FilenameTemplate>,
  /// Set the hot update chunk filename.
  hot_update_chunk_filename: Option<FilenameTemplate>,
  /// Set the hot update global.
  hot_update_global: Option<String>,
  /// Set the library.
  library: Option<LibraryOptions>,
  /// Set the enabled library types.
  enabled_library_types: Option<Vec<LibraryType>>,
  /// Set the enabled chunk loading types.
  enabled_chunk_loading_types: Option<Vec<ChunkLoadingType>>,
  /// Set the enabled wasm loading types.
  enabled_wasm_loading_types: Option<Vec<WasmLoadingType>>,
  /// Set the strict module error handling.
  strict_module_error_handling: Option<bool>,
  /// Set the global object.
  global_object: Option<String>,
  /// Set the import function name.
  import_function_name: Option<String>,
  /// Set the import meta name.
  import_meta_name: Option<String>,
  /// Set the iife.
  iife: Option<bool>,
  /// Set the module.
  module: Option<bool>,
  /// Set the trusted types.
  trusted_types: Option<TrustedTypes>,
  /// Set the source map filename.
  source_map_filename: Option<FilenameTemplate>,
  /// Set the hash function.
  hash_function: Option<HashFunction>,
  /// Set the hash digest.
  hash_digest: Option<HashDigest>,
  /// Set the hash digest length.
  hash_digest_length: Option<usize>,
  /// Set the hash salt.
  hash_salt: Option<HashSalt>,
  /// Set the async chunks.
  async_chunks: Option<bool>,
  /// Set the worker chunk loading.
  worker_chunk_loading: Option<ChunkLoading>,
  /// Set the worker wasm loading.
  worker_wasm_loading: Option<WasmLoading>,
  /// Set the worker public path.
  worker_public_path: Option<String>,
  /// Set the script type.
  script_type: Option<String>,
  /// Set the devtool namespace.
  devtool_namespace: Option<String>,
  /// Set the devtool module filename template.
  devtool_module_filename_template: Option<FilenameTemplate>,
  /// Set the devtool fallback module filename template.
  devtool_fallback_module_filename_template: Option<FilenameTemplate>,
  /// Set the environment.
  environment: Option<Environment>,
  /// Set the compare before emit.
  compare_before_emit: Option<bool>,
}

impl From<OutputOptions> for OutputOptionsBuilder {
  fn from(value: OutputOptions) -> Self {
    OutputOptionsBuilder {
      path: Some(value.path),
      pathinfo: Some(value.pathinfo),
      clean: Some(value.clean),
      public_path: Some(value.public_path),
      asset_module_filename: Some(value.asset_module_filename),
      wasm_loading: Some(value.wasm_loading),
      webassembly_module_filename: Some(value.webassembly_module_filename),
      unique_name: Some(value.unique_name),
      chunk_loading: Some(value.chunk_loading),
      chunk_loading_global: Some(value.chunk_loading_global),
      chunk_load_timeout: Some(value.chunk_load_timeout),
      chunk_format: None,
      charset: Some(value.charset),
      filename: Some(value.filename),
      chunk_filename: Some(value.chunk_filename),
      cross_origin_loading: Some(value.cross_origin_loading),
      css_filename: Some(value.css_filename),
      css_chunk_filename: Some(value.css_chunk_filename),
      hot_update_main_filename: Some(value.hot_update_main_filename),
      hot_update_chunk_filename: Some(value.hot_update_chunk_filename),
      hot_update_global: Some(value.hot_update_global),
      library: value.library,
      enabled_library_types: value.enabled_library_types,
      strict_module_error_handling: Some(value.strict_module_error_handling),
      global_object: Some(value.global_object),
      import_function_name: Some(value.import_function_name),
      import_meta_name: Some(value.import_meta_name),
      iife: Some(value.iife),
      module: Some(value.module),
      trusted_types: value.trusted_types,
      source_map_filename: Some(value.source_map_filename),
      hash_function: Some(value.hash_function),
      hash_digest: Some(value.hash_digest),
      hash_digest_length: Some(value.hash_digest_length),
      hash_salt: Some(value.hash_salt),
      async_chunks: Some(value.async_chunks),
      worker_chunk_loading: Some(value.worker_chunk_loading),
      worker_wasm_loading: Some(value.worker_wasm_loading),
      worker_public_path: Some(value.worker_public_path),
      script_type: Some(value.script_type),
      devtool_namespace: None,
      devtool_module_filename_template: None,
      devtool_fallback_module_filename_template: None,
      environment: Some(value.environment),
      compare_before_emit: Some(value.compare_before_emit),
      enabled_chunk_loading_types: None,
      enabled_wasm_loading_types: None,
    }
  }
}

impl From<&mut OutputOptionsBuilder> for OutputOptionsBuilder {
  fn from(value: &mut OutputOptionsBuilder) -> Self {
    OutputOptionsBuilder {
      path: value.path.take(),
      pathinfo: value.pathinfo.take(),
      clean: value.clean.take(),
      public_path: value.public_path.take(),
      asset_module_filename: value.asset_module_filename.take(),
      wasm_loading: value.wasm_loading.take(),
      webassembly_module_filename: value.webassembly_module_filename.take(),
      unique_name: value.unique_name.take(),
      chunk_loading: value.chunk_loading.take(),
      chunk_loading_global: value.chunk_loading_global.take(),
      chunk_load_timeout: value.chunk_load_timeout.take(),
      chunk_format: value.chunk_format.take(),
      charset: value.charset.take(),
      filename: value.filename.take(),
      chunk_filename: value.chunk_filename.take(),
      cross_origin_loading: value.cross_origin_loading.take(),
      css_filename: value.css_filename.take(),
      css_chunk_filename: value.css_chunk_filename.take(),
      hot_update_main_filename: value.hot_update_main_filename.take(),
      hot_update_chunk_filename: value.hot_update_chunk_filename.take(),
      hot_update_global: value.hot_update_global.take(),
      library: value.library.take(),
      enabled_library_types: value.enabled_library_types.take(),
      enabled_chunk_loading_types: value.enabled_chunk_loading_types.take(),
      enabled_wasm_loading_types: value.enabled_wasm_loading_types.take(),
      strict_module_error_handling: value.strict_module_error_handling.take(),
      global_object: value.global_object.take(),
      import_function_name: value.import_function_name.take(),
      import_meta_name: value.import_meta_name.take(),
      iife: value.iife.take(),
      module: value.module.take(),
      trusted_types: value.trusted_types.take(),
      source_map_filename: value.source_map_filename.take(),
      hash_function: value.hash_function.take(),
      hash_digest: value.hash_digest.take(),
      hash_digest_length: value.hash_digest_length.take(),
      hash_salt: value.hash_salt.take(),
      async_chunks: value.async_chunks.take(),
      worker_chunk_loading: value.worker_chunk_loading.take(),
      worker_wasm_loading: value.worker_wasm_loading.take(),
      worker_public_path: value.worker_public_path.take(),
      script_type: value.script_type.take(),
      devtool_namespace: value.devtool_namespace.take(),
      devtool_module_filename_template: value.devtool_module_filename_template.take(),
      devtool_fallback_module_filename_template: value
        .devtool_fallback_module_filename_template
        .take(),
      environment: value.environment.take(),
      compare_before_emit: value.compare_before_emit.take(),
    }
  }
}

impl OutputOptionsBuilder {
  /// The output directory as an absolute path.
  ///
  /// Default set to `std::env::current_dir().unwrap().join("dist")`.
  pub fn path<V>(&mut self, path: V) -> &mut Self
  where
    V: Into<Utf8PathBuf>,
  {
    self.path = Some(path.into());
    self
  }

  /// Tells Rspack to include comments in bundles with information about the contained modules.
  ///
  /// Default set to `PathInfo::Bool(true)`.
  pub fn pathinfo(&mut self, pathinfo: PathInfo) -> &mut Self {
    self.pathinfo = Some(pathinfo);
    self
  }

  /// Before generating the products, whether delete all files in the output directory.
  ///
  /// Default set to `CleanOptions::CleanAll(false)`.
  pub fn clean(&mut self, clean: CleanOptions) -> &mut Self {
    self.clean = Some(clean);
    self
  }

  /// This option determines the URL prefix of the referenced resource, such as: image, file, etc.
  pub fn public_path(&mut self, public_path: PublicPath) -> &mut Self {
    self.public_path = Some(public_path);
    self
  }

  /// This option determines the name of each output bundle.
  ///
  /// Default set to `"[hash][ext][query]"`.
  pub fn asset_module_filename(&mut self, filename: Filename) -> &mut Self {
    self.asset_module_filename = Some(filename);
    self
  }

  /// This option determines the name of each output wasm bundle.
  ///
  /// Default set to [`WasmLoadingType::Fetch`].
  ///
  /// [`WasmLoadingType`]: rspack_core::options::WasmLoadingType
  pub fn wasm_loading(&mut self, loading: WasmLoading) -> &mut Self {
    self.wasm_loading = Some(loading);
    self
  }

  /// This option determines the name of each output wasm bundle.
  ///
  /// Default set to `"[hash].module.wasm"`.
  pub fn webassembly_module_filename(&mut self, filename: FilenameTemplate) -> &mut Self {
    self.webassembly_module_filename = Some(filename);
    self
  }

  /// A unique name of the Rspack build to avoid multiple Rspack runtimes to conflict when using globals.
  pub fn unique_name(&mut self, name: String) -> &mut Self {
    self.unique_name = Some(name);
    self
  }

  /// This method to load chunks (methods included by default are 'jsonp' (web), 'import' (ESM), 'importScripts' (webworker), 'require' (sync node.js), 'async-node' (async node.js).
  pub fn chunk_loading(&mut self, loading: ChunkLoading) -> &mut Self {
    self.chunk_loading = Some(loading);
    self
  }

  /// The global variable is used by Rspack for loading chunks.
  ///
  /// Determined by output.uniqueName default.
  pub fn chunk_loading_global(&mut self, global: String) -> &mut Self {
    self.chunk_loading_global = Some(global);
    self
  }

  /// The Number of milliseconds before chunk request timed out.
  ///
  /// Default set to `120000`.
  pub fn chunk_load_timeout(&mut self, timeout: u32) -> &mut Self {
    self.chunk_load_timeout = Some(timeout);
    self
  }

  /// The format of chunks (formats included by default are 'array-push' (web/webworker), 'commonjs' (node.js), 'module' (ESM).
  pub fn chunk_format(&mut self, chunk_format: String) -> &mut Self {
    self.chunk_format = Some(chunk_format);
    self
  }

  /// Add charset="utf-8" to the HTML <script> tag.
  ///
  /// Default set to `true`.
  pub fn charset(&mut self, charset: bool) -> &mut Self {
    self.charset = Some(charset);
    self
  }

  /// Set the name of each asset resource output bundle.
  pub fn filename(&mut self, filename: Filename) -> &mut Self {
    self.filename = Some(filename);
    self
  }

  /// Set the name of non-initial chunk files.
  pub fn chunk_filename(&mut self, filename: Filename) -> &mut Self {
    self.chunk_filename = Some(filename);
    self
  }

  /// Set the [crossorigin attribute](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/script) for dynamically loaded chunks.
  pub fn cross_origin_loading(&mut self, loading: CrossOriginLoading) -> &mut Self {
    self.cross_origin_loading = Some(loading);
    self
  }

  /// Set the name of CSS output files on disk.
  pub fn css_filename(&mut self, filename: Filename) -> &mut Self {
    self.css_filename = Some(filename);
    self
  }

  /// Set the name of non-initial CSS output files on disk.
  pub fn css_chunk_filename(&mut self, filename: Filename) -> &mut Self {
    self.css_chunk_filename = Some(filename);
    self
  }

  /// Customize the main hot update filename. [fullhash] and [runtime] are available as placeholder.
  ///
  /// Default set to `"[runtime].[fullhash].hot-update.json"`.
  pub fn hot_update_main_filename(&mut self, filename: FilenameTemplate) -> &mut Self {
    self.hot_update_main_filename = Some(filename);
    self
  }

  /// Customize the filenames of hot update chunks.
  ///
  /// Default set to `"[id].[fullhash].hot-update.js"`.
  pub fn hot_update_chunk_filename(&mut self, filename: FilenameTemplate) -> &mut Self {
    self.hot_update_chunk_filename = Some(filename);
    self
  }

  /// Set the global variable is used by Rspack for loading hot updates.
  ///
  /// Determined by output.uniqueName default.
  pub fn hot_update_global(&mut self, global: String) -> &mut Self {
    self.hot_update_global = Some(global);
    self
  }

  /// Set the name of each output bundle.
  pub fn library(&mut self, library: LibraryOptions) -> &mut Self {
    self.library = Some(library);
    self
  }

  /// Set list of library types enabled for use by entry points.
  pub fn enabled_library_types(&mut self, types: Vec<LibraryType>) -> &mut Self {
    self.enabled_library_types = Some(types);
    self
  }

  /// Set list of chunk loading types enabled for use by entry points.
  pub fn enabled_chunk_loading_types(&mut self, types: Vec<ChunkLoadingType>) -> &mut Self {
    self.enabled_chunk_loading_types = Some(types);
    self
  }

  /// Set list of wasm loading types enabled for use by entry points.
  pub fn enabled_wasm_loading_types(&mut self, types: Vec<WasmLoadingType>) -> &mut Self {
    self.enabled_wasm_loading_types = Some(types);
    self
  }

  /// Set whether to enable strict module error handling.
  pub fn strict_module_error_handling(&mut self, strict: bool) -> &mut Self {
    self.strict_module_error_handling = Some(strict);
    self
  }

  /// Set global object that indicates what global object will be used to mount the library.
  pub fn global_object(&mut self, object: String) -> &mut Self {
    self.global_object = Some(object);
    self
  }

  /// Set the name of the import function.
  pub fn import_function_name(&mut self, name: String) -> &mut Self {
    self.import_function_name = Some(name);
    self
  }

  /// Set the name of the import meta.
  pub fn import_meta_name(&mut self, name: String) -> &mut Self {
    self.import_meta_name = Some(name);
    self
  }

  /// Set whether to tell Rspack to add IIFE wrapper around emitted code.
  pub fn iife(&mut self, iife: bool) -> &mut Self {
    self.iife = Some(iife);
    self
  }

  /// Set whether to output JavaScript files as module type.
  pub fn module(&mut self, module: bool) -> &mut Self {
    self.module = Some(module);
    self
  }

  /// Set controls [Trusted Types](https://web.dev/articles/trusted-types) compatibility.
  pub fn trusted_types(&mut self, trusted_types: TrustedTypes) -> &mut Self {
    self.trusted_types = Some(trusted_types);
    self
  }

  /// Set the name of the source map file.
  pub fn source_map_filename(&mut self, filename: FilenameTemplate) -> &mut Self {
    self.source_map_filename = Some(filename);
    self
  }

  /// Set the hash function.
  pub fn hash_function(&mut self, function: HashFunction) -> &mut Self {
    self.hash_function = Some(function);
    self
  }

  /// Set the hash digest.
  pub fn hash_digest(&mut self, digest: HashDigest) -> &mut Self {
    self.hash_digest = Some(digest);
    self
  }

  /// Set the hash digest length.
  pub fn hash_digest_length(&mut self, length: usize) -> &mut Self {
    self.hash_digest_length = Some(length);
    self
  }

  /// Set the hash salt.
  pub fn hash_salt(&mut self, salt: HashSalt) -> &mut Self {
    self.hash_salt = Some(salt);
    self
  }

  /// Set whether to enable async chunks.
  pub fn async_chunks(&mut self, async_chunks: bool) -> &mut Self {
    self.async_chunks = Some(async_chunks);
    self
  }

  /// Set the chunk loading type for worker.
  pub fn worker_chunk_loading(&mut self, loading: ChunkLoading) -> &mut Self {
    self.worker_chunk_loading = Some(loading);
    self
  }

  /// Set the wasm loading type for worker.
  pub fn worker_wasm_loading(&mut self, loading: WasmLoading) -> &mut Self {
    self.worker_wasm_loading = Some(loading);
    self
  }

  /// Set the public path for Worker.
  pub fn worker_public_path(&mut self, path: String) -> &mut Self {
    self.worker_public_path = Some(path);
    self
  }

  /// Set the type of the script.
  pub fn script_type(&mut self, script_type: String) -> &mut Self {
    self.script_type = Some(script_type);
    self
  }

  /// Set the namespace of the devtool.
  pub fn devtool_namespace(&mut self, namespace: String) -> &mut Self {
    self.devtool_namespace = Some(namespace);
    self
  }

  /// Set the template of the devtool module filename.
  pub fn devtool_module_filename_template(&mut self, filename: FilenameTemplate) -> &mut Self {
    self.devtool_module_filename_template = Some(filename);
    self
  }

  /// Set the template of the devtool fallback module filename.
  pub fn devtool_fallback_module_filename_template(
    &mut self,
    filename: FilenameTemplate,
  ) -> &mut Self {
    self.devtool_fallback_module_filename_template = Some(filename);
    self
  }

  /// Set the environment.
  pub fn environment(&mut self, environment: Environment) -> &mut Self {
    self.environment = Some(environment);
    self
  }

  /// Set whether to compare the emitted code before emitting.
  pub fn compare_before_emit(&mut self, compare: bool) -> &mut Self {
    self.compare_before_emit = Some(compare);
    self
  }

  /// Build [`OutputOptions`] from builder.
  ///
  /// [`OutputOptions`]: rspack_core::options::OutputOptions
  #[allow(clippy::too_many_arguments, clippy::fn_params_excessive_bools)]
  fn build(
    &mut self,
    builder_context: &mut BuilderContext,
    context: &Context,
    output_module: bool,
    target_properties: Option<&TargetProperties>,
    is_affected_by_browserslist: bool,
    development: bool,
    entry: &IndexMap<String, EntryDescription>,
    _future_defaults: bool,
  ) -> Result<OutputOptions> {
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

    let cross_origin_loading = d!(
      self.cross_origin_loading.take(),
      CrossOriginLoading::Disable
    );

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

    let chunk_load_timeout = d!(self.chunk_load_timeout.take(), 120_000);

    let charset = d!(self.charset.take(), true);

    let hot_update_global = f!(self.hot_update_global.take(), || {
      format!(
        "webpackHotUpdate{}",
        rspack_core::to_identifier(&unique_name)
      )
    });

    let chunk_format = if let Some(chunk_format) = self.chunk_format.take() {
      chunk_format
    } else if let Some(tp) = tp {
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
          return Err(BuilderError::Option("output.chunk_format".to_string(), format!("For the selected environment is no default ESM chunk format available:\nESM exports can be chosen when 'import()' is available.\nJSONP Array push can be chosen when 'document' is available.\n{help_message}")).into());
        }
      } else if tp.document() {
        "array-push".to_string()
      } else if tp.require() || tp.node_builtins() {
        "commonjs".to_string()
      } else if tp.import_scripts() {
        "array-push".to_string()
      } else {
        return Err(BuilderError::Option("output.chunk_format".to_string(), format!("For the selected environment is no default script chunk format available:\nJSONP Array push can be chosen when 'document' or 'importScripts' is available.\nCommonJs exports can be chosen when 'require' or node builtins are available.\n{help_message}")).into());
      }
    } else {
      return Err(
        BuilderError::Option(
          "output.chunk_format".to_string(),
          "Chunk format can't be selected by default when no target is specified".to_string(),
        )
        .into(),
      );
    };

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

    let strict_module_error_handling = d!(self.strict_module_error_handling.take(), false);
    let import_function_name = f!(self.import_function_name.take(), || "import".into());
    let import_meta_name = f!(self.import_meta_name.take(), || "import.meta".into());
    let iife = d!(self.iife.take(), !output_module);
    let module = d!(self.module.take(), output_module);
    let source_map_filename = f!(self.source_map_filename.take(), || "[file].map[query]"
      .into());
    let hash_function = d!(self.hash_function.take(), HashFunction::Xxhash64);
    let hash_digest = d!(self.hash_digest.take(), HashDigest::Hex);
    let hash_digest_length = d!(self.hash_digest_length.take(), 16);
    let hash_salt = d!(self.hash_salt.take(), HashSalt::None);
    let async_chunks = d!(self.async_chunks.take(), true);

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
          WasmLoading::Enable(WasmLoadingType::AsyncNode)
        } else {
          WasmLoading::Disable
        }
      } else {
        WasmLoading::Disable
      }
    });

    let webassembly_module_filename = f!(self.webassembly_module_filename.take(), || {
      "[hash].module.wasm".into()
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
      for (_, desc) in entry.iter() {
        if let Some(library) = &desc.library {
          enabled_library_types.push(library.library_type.clone());
        }
      }
      enabled_library_types
    });
    for ty in enabled_library_types.iter() {
      builder_context
        .plugins
        .push(BuiltinPluginOptions::EnableLibraryPlugin(ty.clone()));
    }

    let enabled_chunk_loading_types = f!(
      self
        .enabled_chunk_loading_types
        .take()
        .map(|types| { types.into_iter().collect::<HashSet<_>>() }),
      || {
        let mut enabled_chunk_loading_types = HashSet::default();
        if let ChunkLoading::Enable(ty) = &chunk_loading {
          enabled_chunk_loading_types.insert(ty.clone());
        }
        if let ChunkLoading::Enable(ty) = &worker_chunk_loading {
          enabled_chunk_loading_types.insert(ty.clone());
        }
        for (_, desc) in entry.iter() {
          if let Some(ChunkLoading::Enable(ty)) = &desc.chunk_loading {
            enabled_chunk_loading_types.insert(ty.clone());
          }
        }
        enabled_chunk_loading_types
      }
    );
    for ty in enabled_chunk_loading_types {
      builder_context
        .plugins
        .push(BuiltinPluginOptions::EnableChunkLoadingPlugin(ty));
    }

    let enabled_wasm_loading_types = f!(
      self
        .enabled_wasm_loading_types
        .take()
        .map(|types| { types.into_iter().collect::<HashSet<_>>() }),
      || {
        let mut enabled_wasm_loading_types = HashSet::default();
        if let WasmLoading::Enable(ty) = wasm_loading {
          enabled_wasm_loading_types.insert(ty);
        }
        if let WasmLoading::Enable(ty) = worker_wasm_loading {
          enabled_wasm_loading_types.insert(ty);
        }
        // for (_, desc) in entry.iter() {
        //   if let Some(wasm_loading) = &desc.wasm_loading {
        //     enabled_wasm_loading_types.push(wasm_loading.clone());
        //   }
        // }
        enabled_wasm_loading_types
      }
    );

    for ty in enabled_wasm_loading_types {
      builder_context
        .plugins
        .push(BuiltinPluginOptions::EnableWasmLoadingPlugin(ty));
    }

    let script_type = f!(self.script_type.take(), || {
      if output_module {
        "module".to_string()
      } else {
        String::new()
      }
    });

    macro_rules! optimistic {
      ($tp:expr) => {
        matches!($tp, Some(true)) || $tp.is_none()
      };
    }
    macro_rules! conditionally_optimistic {
      ($tp:expr, $condition:expr) => {
        ($tp.is_none() && $condition) || $tp.unwrap_or_default()
      };
    }

    let mut environment = f!(self.environment.take(), Environment::default);
    environment.global_this = tp.and_then(|t| t.global_this);
    environment.big_int_literal = tp.map(|t| optimistic!(t.big_int_literal));
    environment.r#const = tp.map(|t| optimistic!(t.r#const));
    environment.arrow_function = tp.map(|t| optimistic!(t.arrow_function));
    environment.async_function = tp.map(|t| optimistic!(t.async_function));
    environment.for_of = tp.map(|t| optimistic!(t.for_of));
    environment.destructuring = tp.map(|t| optimistic!(t.destructuring));
    environment.optional_chaining = tp.map(|t| optimistic!(t.optional_chaining));
    environment.node_prefix_for_core_modules =
      tp.map(|t| optimistic!(t.node_prefix_for_core_modules));
    environment.template_literal = tp.map(|t| optimistic!(t.template_literal));
    environment.dynamic_import =
      tp.map(|t| conditionally_optimistic!(t.dynamic_import, output_module));
    environment.dynamic_import_in_worker =
      tp.map(|t| conditionally_optimistic!(t.dynamic_import_in_worker, output_module));
    environment.module = tp.map(|t| conditionally_optimistic!(t.module, output_module));
    environment.document = tp.map(|t| optimistic!(t.document));

    Ok(OutputOptions {
      path,
      pathinfo,
      clean,
      asset_module_filename,
      public_path,
      wasm_loading,
      webassembly_module_filename,
      unique_name,
      chunk_loading,
      chunk_loading_global,
      chunk_load_timeout,
      charset,
      filename,
      chunk_filename,
      cross_origin_loading,
      css_filename,
      css_chunk_filename,
      hot_update_main_filename,
      hot_update_chunk_filename,
      hot_update_global,
      library: self.library.take(),
      enabled_library_types: Some(enabled_library_types),
      strict_module_error_handling,
      global_object,
      import_function_name,
      import_meta_name,
      iife,
      module,
      trusted_types: self.trusted_types.take(),
      source_map_filename,
      hash_function,
      hash_digest,
      hash_digest_length,
      hash_salt,
      async_chunks,
      worker_chunk_loading,
      worker_wasm_loading,
      worker_public_path: self.worker_public_path.take().unwrap_or_default(),
      script_type,
      environment,
      compare_before_emit: self.compare_before_emit.take().unwrap_or(true),
    })
  }
}

/// Builder used to build options for optimization plugins.
///
/// See [`OptimizationOptions`] for more details.
///
/// [`OptimizationOptions`]: rspack_core::options::Optimization
#[derive(Debug, Default)]
pub struct OptimizationOptionsBuilder {
  /// Detect and remove modules from chunks these modules are already included in all parents.
  remove_available_modules: Option<bool>,
  /// Remove empty chunks generated in the compilation.
  remove_empty_chunks: Option<bool>,
  /// Merge chunks which contain the same modules.
  merge_duplicate_chunks: Option<bool>,
  /// Which algorithm to use when choosing module ids.
  module_ids: Option<String>,
  /// Which algorithm to use when choosing chunk ids.
  chunk_ids: Option<String>,
  /// Whether to enable minimize.
  minimize: Option<bool>,
  /// Minimizer.
  minimizer: Option<Vec<BuiltinPluginOptions>>,
  /// Whether to enable side effects.
  side_effects: Option<SideEffectOption>,
  /// Whether to enable provided exports.
  provided_exports: Option<bool>,
  /// Whether to enable used exports.
  used_exports: Option<UsedExportsOption>,
  /// Whether to enable inner graph.
  inner_graph: Option<bool>,
  /// Whether to enable mangle exports.
  mangle_exports: Option<MangleExportsOption>,
  /// Whether to enable concatenate modules.
  concatenate_modules: Option<bool>,
  /// Whether to enable real content hash.
  real_content_hash: Option<bool>,
  /// Whether to enable avoid entry iife.
  avoid_entry_iife: Option<bool>,
  /// Node env.
  node_env: Option<String>,
  /// Whether to emit on errors.
  emit_on_errors: Option<bool>,
  /// Runtime chunk.
  runtime_chunk: Option<rspack_plugin_runtime_chunk::RuntimeChunkOptions>,
  // TODO: split chunks
}

impl From<Optimization> for OptimizationOptionsBuilder {
  fn from(value: Optimization) -> Self {
    OptimizationOptionsBuilder {
      remove_available_modules: Some(value.remove_available_modules),
      side_effects: Some(value.side_effects),
      provided_exports: Some(value.provided_exports),
      used_exports: Some(value.used_exports),
      inner_graph: Some(value.inner_graph),
      mangle_exports: Some(value.mangle_exports),
      concatenate_modules: Some(value.concatenate_modules),
      avoid_entry_iife: Some(value.avoid_entry_iife),
      remove_empty_chunks: None,
      merge_duplicate_chunks: None,
      module_ids: None,
      chunk_ids: None,
      minimize: None,
      minimizer: None,
      real_content_hash: None,
      node_env: None,
      emit_on_errors: None,
      runtime_chunk: None,
    }
  }
}

impl From<&mut OptimizationOptionsBuilder> for OptimizationOptionsBuilder {
  fn from(value: &mut OptimizationOptionsBuilder) -> Self {
    OptimizationOptionsBuilder {
      remove_available_modules: value.remove_available_modules.take(),
      remove_empty_chunks: value.remove_empty_chunks.take(),
      merge_duplicate_chunks: value.merge_duplicate_chunks.take(),
      module_ids: value.module_ids.take(),
      chunk_ids: value.chunk_ids.take(),
      minimize: value.minimize.take(),
      minimizer: value.minimizer.take(),
      side_effects: value.side_effects.take(),
      provided_exports: value.provided_exports.take(),
      used_exports: value.used_exports.take(),
      inner_graph: value.inner_graph.take(),
      mangle_exports: value.mangle_exports.take(),
      concatenate_modules: value.concatenate_modules.take(),
      real_content_hash: value.real_content_hash.take(),
      avoid_entry_iife: value.avoid_entry_iife.take(),
      node_env: value.node_env.take(),
      emit_on_errors: value.emit_on_errors.take(),
      runtime_chunk: value.runtime_chunk.take(),
    }
  }
}

impl OptimizationOptionsBuilder {
  /// Set whether to detect and remove modules from chunks these modules are already included in all parents.
  pub fn remove_available_modules(&mut self, value: bool) -> &mut Self {
    self.remove_available_modules = Some(value);
    self
  }

  /// Set whether to remove empty chunks generated in the compilation.
  pub fn remove_empty_chunks(&mut self, value: bool) -> &mut Self {
    self.remove_empty_chunks = Some(value);
    self
  }

  /// Set whether to merge chunks which contain the same modules.
  pub fn merge_duplicate_chunks(&mut self, value: bool) -> &mut Self {
    self.merge_duplicate_chunks = Some(value);
    self
  }

  /// Set which algorithm to use when choosing module ids.
  pub fn module_ids(&mut self, value: String) -> &mut Self {
    self.module_ids = Some(value);
    self
  }

  /// Set which algorithm to use when choosing chunk ids.
  pub fn chunk_ids(&mut self, value: String) -> &mut Self {
    self.chunk_ids = Some(value);
    self
  }

  /// Set whether to enable minimize.
  ///
  /// Default set to `true` in production mode.
  /// Default set to `false` in development mode.
  pub fn minimize(&mut self, value: bool) -> &mut Self {
    self.minimize = Some(value);
    self
  }

  /// Set the minimizer.
  pub fn minimizer(&mut self, value: Vec<BoxPlugin>) -> &mut Self {
    self.minimizer = Some(
      value
        .into_iter()
        .map(BuiltinPluginOptions::AnyMinimizerRspackPlugin)
        .collect(),
    );
    self
  }

  /// Set whether to enable side effects.
  ///
  /// Default set to `true` in production mode.
  /// Default set to `false` in development mode.
  pub fn side_effects(&mut self, value: SideEffectOption) -> &mut Self {
    self.side_effects = Some(value);
    self
  }

  /// Set whether to enable provided exports.
  ///
  /// After enabling, Rspack will analyze which exports the module provides, including re-exported modules.
  ///
  /// Default set to `true`.
  pub fn provided_exports(&mut self, value: bool) -> &mut Self {
    self.provided_exports = Some(value);
    self
  }

  /// Set whether to enable used exports.
  ///
  /// Default set to `true` in production mode.
  /// Default set to `false` in development mode.
  pub fn used_exports(&mut self, value: UsedExportsOption) -> &mut Self {
    self.used_exports = Some(value);
    self
  }

  /// Set whether to enable inner graph.
  ///
  /// Default set to `true` in production mode.
  /// Default set to `false` in development mode.
  pub fn inner_graph(&mut self, value: bool) -> &mut Self {
    self.inner_graph = Some(value);
    self
  }

  /// Set whether to enable mangle exports.
  ///
  /// Default set to `deterministic` in production mode.
  /// Default set to `false` in development mode.
  pub fn mangle_exports(&mut self, value: MangleExportsOption) -> &mut Self {
    self.mangle_exports = Some(value);
    self
  }

  /// Set whether to enable concatenate modules.
  ///
  /// Default set to `true` in production mode.
  /// Default set to `false` in development mode.
  pub fn concatenate_modules(&mut self, value: bool) -> &mut Self {
    self.concatenate_modules = Some(value);
    self
  }

  /// Set whether to enable real content hash.
  ///
  /// Default set to `true` in production mode.
  /// Default set to `false` in development mode.
  pub fn real_content_hash(&mut self, value: bool) -> &mut Self {
    self.real_content_hash = Some(value);
    self
  }

  /// Set whether to enable avoid entry iife.
  ///
  /// Default set to `false`.
  pub fn avoid_entry_iife(&mut self, value: bool) -> &mut Self {
    self.avoid_entry_iife = Some(value);
    self
  }

  /// Set the node env.
  pub fn node_env<V>(&mut self, value: V) -> &mut Self
  where
    V: Into<String>,
  {
    self.node_env = Some(serde_json::json!(value.into()).to_string());
    self
  }

  /// Set whether to emit on errors.
  ///
  /// Default set to `true` in production mode.
  /// Default set to `false` in development mode.
  pub fn emit_on_errors(&mut self, value: bool) -> &mut Self {
    self.emit_on_errors = Some(value);
    self
  }

  /// Set the runtime chunk.
  pub fn runtime_chunk(
    &mut self,
    value: rspack_plugin_runtime_chunk::RuntimeChunkOptions,
  ) -> &mut Self {
    self.runtime_chunk = Some(value);
    self
  }

  /// Build [`Optimization`] from options.
  ///
  /// [`Optimization`]: rspack_core::options::Optimization
  fn build(
    &mut self,
    builder_context: &mut BuilderContext,
    development: bool,
    production: bool,
    _css: bool,
  ) -> Result<Optimization> {
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
        return Err(
          BuilderError::Option(
            "optimization.module_ids".to_string(),
            format!("{module_ids} is not implemented"),
          )
          .into(),
        );
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
        return Err(
          BuilderError::Option(
            "optimization.chunk_ids".to_string(),
            format!("{chunk_ids} is not implemented"),
          )
          .into(),
        );
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

    let node_env = self.node_env.take().or_else(|| {
      if production {
        Some("production".to_string())
      } else if development {
        Some("development".to_string())
      } else {
        None
      }
    });
    if let Some(node_env) = node_env {
      builder_context
        .plugins
        .push(BuiltinPluginOptions::DefinePlugin(
          [(
            "process.env.NODE_ENV".to_string(),
            format!("{}", json!(node_env)).into(),
          )]
          .into(),
        ));
    }

    Ok(Optimization {
      remove_available_modules,
      side_effects,
      provided_exports,
      used_exports,
      inner_graph,
      mangle_exports,
      concatenate_modules,
      avoid_entry_iife,
      real_content_hash,
    })
  }
}

/// Builder used to build [`Experiments`].
///
/// [`Experiments`]: rspack_core::options::Experiments
#[derive(Debug, Default)]
pub struct ExperimentsBuilder {
  /// Whether to enable module layers feature.  
  layers: Option<bool>,
  /// Incremental passes.
  incremental: Option<IncrementalPasses>,
  /// Whether to enable top level await.
  top_level_await: Option<bool>,
  /// Rspack future.
  rspack_future: Option<RspackFuture>,
  /// Cache options.
  cache: Option<ExperimentCacheOptions>,
  /// Whether to enable output module.
  output_module: Option<bool>,
  /// Whether to enable future defaults.
  future_defaults: Option<bool>,
  /// Whether to enable css.
  css: Option<bool>,
  /// Whether to enable parallel code splitting.
  parallel_code_splitting: Option<bool>,
  /// Whether to enable async web assembly.
  async_web_assembly: Option<bool>,
  // TODO: lazy compilation
}

impl From<Experiments> for ExperimentsBuilder {
  fn from(value: Experiments) -> Self {
    ExperimentsBuilder {
      layers: Some(value.layers),
      incremental: Some(value.incremental),
      top_level_await: Some(value.top_level_await),
      rspack_future: Some(value.rspack_future),
      cache: Some(value.cache),
      parallel_code_splitting: Some(value.parallel_code_splitting),
      output_module: None,
      future_defaults: None,
      css: None,
      async_web_assembly: None,
    }
  }
}

impl From<&mut ExperimentsBuilder> for ExperimentsBuilder {
  fn from(value: &mut ExperimentsBuilder) -> Self {
    ExperimentsBuilder {
      layers: value.layers.take(),
      incremental: value.incremental.take(),
      top_level_await: value.top_level_await.take(),
      rspack_future: value.rspack_future.take(),
      cache: value.cache.take(),
      output_module: value.output_module.take(),
      future_defaults: value.future_defaults.take(),
      css: value.css.take(),
      parallel_code_splitting: value.parallel_code_splitting.take(),
      async_web_assembly: value.async_web_assembly.take(),
    }
  }
}

impl ExperimentsBuilder {
  /// Set whether to enable layers.
  pub fn layers(&mut self, layers: bool) -> &mut Self {
    self.layers = Some(layers);
    self
  }

  /// Set the incremental passes.
  pub fn incremental(&mut self, incremental: IncrementalPasses) -> &mut Self {
    self.incremental = Some(incremental);
    self
  }

  /// Set whether to enable top level await.
  pub fn top_level_await(&mut self, top_level_await: bool) -> &mut Self {
    self.top_level_await = Some(top_level_await);
    self
  }

  /// Set the cache options.
  pub fn cache(&mut self, cache: ExperimentCacheOptions) -> &mut Self {
    self.cache = Some(cache);
    self
  }

  /// Set whether to enable future defaults.
  pub fn future_defaults(&mut self, future_defaults: bool) -> &mut Self {
    self.future_defaults = Some(future_defaults);
    self
  }

  /// Set whether to enable css.
  pub fn css(&mut self, css: bool) -> &mut Self {
    self.css = Some(css);
    self
  }

  /// Set whether to enable async web assembly.
  pub fn async_web_assembly(&mut self, async_web_assembly: bool) -> &mut Self {
    self.async_web_assembly = Some(async_web_assembly);
    self
  }

  /// Set whether to enable parallel code splitting.
  pub fn parallel_code_splitting(&mut self, parallel_code_splitting: bool) -> &mut Self {
    self.parallel_code_splitting = Some(parallel_code_splitting);
    self
  }

  /// Build [`Experiments`] from options.
  ///
  /// [`Experiments`]: rspack_core::options::Experiments
  fn build(
    &mut self,
    _builder_context: &mut BuilderContext,
    development: bool,
    production: bool,
  ) -> Result<Experiments> {
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

    let parallel_code_splitting = d!(self.parallel_code_splitting, false);

    Ok(Experiments {
      layers,
      incremental,
      top_level_await,
      rspack_future,
      parallel_code_splitting,
      cache,
    })
  }
}

#[cfg(test)]
mod test {
  use enum_tag::EnumTag;

  use super::*;

  #[test]
  fn default() {
    let _ = CompilerOptions::builder().build(&mut Default::default());
  }

  #[test]
  fn builtin_plugin_order() {
    let mut context: BuilderContext = Default::default();
    let compiler_options = CompilerOptions::builder()
      .mode(Mode::Production)
      .target(vec!["web".to_string()])
      .build(&mut context)
      .unwrap();
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

    let plugins = context.take_plugins(&compiler_options);
    assert!(!plugins.is_empty());
  }

  #[test]
  fn mutable_builder_into_owned_builder() {
    let _ = CompilerOptions::builder()
      .optimization(OptimizationOptionsBuilder::default().node_env("development".to_string()))
      .output(OutputOptionsBuilder::default().charset(true))
      .experiments(ExperimentsBuilder::default().future_defaults(true))
      .module(ModuleOptionsBuilder::default().no_parse(ModuleNoParseRules::Rules(vec![])))
      .node(NodeOptionBuilder::default().dirname(NodeDirnameOption::EvalOnly))
      .build(&mut Default::default());
  }

  #[test]
  #[should_panic]
  #[allow(unreachable_code, unused_variables)]
  fn use_options_directly() {
    let optimization: Optimization = todo!();
    let output: OutputOptions = todo!();
    let experiments: Experiments = todo!();
    let module: ModuleOptions = todo!();
    let node: Option<NodeOption> = todo!();
    Compiler::builder()
      .optimization(optimization)
      .output(output)
      .experiments(experiments)
      .module(module)
      .node(node);
  }
}
