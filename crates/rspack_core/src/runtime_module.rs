use std::{
  fmt::Debug,
  ops::{BitOr, BitOrAssign},
  sync::Arc,
};

use async_trait::async_trait;
use futures::future::BoxFuture;
use rspack_cacheable::cacheable;
use rspack_collections::Identifier;
use rspack_error::Result;
use rspack_hash::{RspackHash, RspackHashDigest, RspackHasher};
use rspack_sources::{BoxSource, OriginalSource, RawStringSource, Source, SourceExt};
use rspack_util::{ext::AsAny, source_map::SourceMapKind};
use tokio::sync::OnceCell;

use crate::{
  ChunkUkey, CodeGenerationResultBuilder, Compilation, Module, ModuleCodeGenerationContext,
  RuntimeCodeTemplate, RuntimeGlobals, RuntimeGlobalsRenderMode, RuntimeTemplate, SourceType,
  runtime_mode::RuntimeMode, runtime_module_transform::transform_runtime_module_source,
};

pub struct RuntimeModuleGenerateContext<'a> {
  pub compilation: &'a Compilation,
  pub runtime_template: &'a RuntimeCodeTemplate,
}

/// Immutable data retained by HMR records for the next runtime update.
/// The producing runtime module defines the representation of this data.
pub trait RuntimeModuleUpdateData: Debug + AsAny + Send + Sync {}

#[derive(Debug)]
pub struct RuntimeModuleRecord {
  pub hash: RspackHashDigest,
  pub update_data: Option<Arc<dyn RuntimeModuleUpdateData>>,
  pub has_custom_source: bool,
}

pub fn runtime_module_owned_define_fields(
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
) -> RuntimeGlobals {
  compilation
    .build_chunk_graph_artifact
    .chunk_graph
    .get_chunk_runtime_modules_iterable(chunk_ukey)
    .fold(RuntimeGlobals::default(), |fields, runtime_module_id| {
      let runtime_module = compilation
        .runtime_modules
        .get(runtime_module_id)
        .expect("should have runtime module");
      if runtime_module.get_custom_source().has_override()
        || runtime_module.get_constructor_name() == "RuntimeModuleFromJs"
      {
        return fields;
      }
      let runtime_requirements = runtime_module.runtime_requirements(compilation);
      let define_fields = runtime_requirements
        .define
        .difference(RuntimeGlobals::STARTUP);
      fields | define_fields
    })
}

#[derive(Debug, Default, Clone, Copy)]
pub struct RuntimeModuleRuntimeRequirements {
  pub dependencies: RuntimeGlobals,
  pub weak: RuntimeGlobals,
  pub define: RuntimeGlobals,
  pub force_context: RuntimeGlobals,
}

impl RuntimeModuleRuntimeRequirements {
  pub fn lexical_requirements(&self) -> RuntimeGlobals {
    self.dependencies | self.weak | self.define | self.force_context
  }
}

impl BitOr for RuntimeModuleRuntimeRequirements {
  type Output = Self;

  fn bitor(self, rhs: Self) -> Self::Output {
    Self {
      dependencies: self.dependencies | rhs.dependencies,
      weak: self.weak | rhs.weak,
      define: self.define | rhs.define,
      force_context: self.force_context | rhs.force_context,
    }
  }
}

impl BitOrAssign for RuntimeModuleRuntimeRequirements {
  fn bitor_assign(&mut self, rhs: Self) {
    self.dependencies.insert(rhs.dependencies);
    self.weak.insert(rhs.weak);
    self.define.insert(rhs.define);
    self.force_context.insert(rhs.force_context);
  }
}

/// Source adaptation state, cloned with its owning runtime module.
/// Cloning copies the original and transformed source strings.
#[cacheable]
#[derive(Debug, Default, Clone)]
pub struct RuntimeModuleCustomSource {
  /// Original source exposed to runtime module hooks.
  pub source: Option<String>,
  pub(crate) transformed_source: Option<String>,
  pub(crate) generated: bool,
  pub runtime_requirements: RuntimeGlobals,
}

impl RuntimeModuleCustomSource {
  pub fn has_override(&self) -> bool {
    self.source.is_some() && !self.generated
  }

  fn new(source: String, render_mode: RuntimeGlobalsRenderMode) -> Result<Self> {
    if render_mode.is_legacy() {
      return Ok(Self {
        source: Some(source),
        ..Default::default()
      });
    }
    transform_runtime_module_source(source, render_mode)
  }
}

#[cacheable]
#[derive(Debug, Default, Clone)]
pub struct RuntimeModuleCommon {
  pub id: Identifier,
  pub chunk: Option<ChunkUkey>,
  pub source_map_kind: SourceMapKind,
  pub render_mode: RuntimeGlobalsRenderMode,
  pub custom_source: RuntimeModuleCustomSource,
  #[cacheable(with=rspack_cacheable::with::Skip)]
  pub cached_generated_code: Arc<OnceCell<BoxSource>>,
}

impl RuntimeModuleCommon {
  pub fn with_default(runtime_template: &RuntimeTemplate, type_name: &str) -> Self {
    Self {
      source_map_kind: SourceMapKind::empty(),
      render_mode: runtime_template.render_mode().runtime_module_render_mode(),
      custom_source: Default::default(),
      cached_generated_code: Default::default(),
      chunk: None,
      id: runtime_template.create_runtime_module_identifier(type_name),
    }
  }

  pub fn with_name(runtime_template: &RuntimeTemplate, name: &str) -> Self {
    Self {
      source_map_kind: SourceMapKind::empty(),
      render_mode: runtime_template.render_mode().runtime_module_render_mode(),
      custom_source: Default::default(),
      cached_generated_code: Default::default(),
      chunk: None,
      id: runtime_template.create_custom_runtime_module_identifier(name),
    }
  }

  pub fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }

  pub fn id(&self) -> &Identifier {
    &self.id
  }

  pub fn chunk(&self) -> Option<ChunkUkey> {
    self.chunk
  }

  pub fn set_custom_source(&mut self, source: String) -> Result<()> {
    let source = RuntimeModuleCustomSource::new(source, self.render_mode)
      .map_err(|error| error.wrap_err(format!("in runtime module {}", self.id)))?;
    self.custom_source = source;
    if !self.render_mode.is_legacy() {
      self.cached_generated_code = Default::default();
    }
    Ok(())
  }

  pub fn get_custom_source(&self) -> &RuntimeModuleCustomSource {
    &self.custom_source
  }

  pub fn set_generated_source(&mut self, source: String) -> Result<()> {
    self.set_custom_source(source)?;
    self.custom_source.generated = true;
    Ok(())
  }

  pub async fn generate_with_custom(
    &self,
    module: &dyn RuntimeModule,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> Result<String> {
    let custom_source = &self.custom_source;
    // Hot updates access the owning runtime instead of closing over copied bindings.
    let render_mode = if module.is_custom()
      && self.render_mode == RuntimeGlobalsRenderMode::RspackLexical
      && context.runtime_template.render_mode() == RuntimeGlobalsRenderMode::RspackContext
    {
      RuntimeGlobalsRenderMode::RspackContext
    } else {
      self.render_mode
    };
    if let Some(source) = &custom_source.source
      && (custom_source.has_override() || (!module.full_hash() && !module.dependent_hash()))
    {
      if render_mode != self.render_mode {
        return Ok(
          RuntimeModuleCustomSource::new(source.clone(), render_mode)
            .map_err(|error| error.wrap_err(format!("in runtime module {}", self.id)))?
            .transformed_source
            .expect("transformed custom runtime source"),
        );
      }
      return Ok(
        custom_source
          .transformed_source
          .as_ref()
          .unwrap_or(source)
          .clone(),
      );
    }
    let source = module.generate(context).await?;
    if module.is_custom() && !render_mode.is_legacy() {
      let source = RuntimeModuleCustomSource::new(source, render_mode)
        .map_err(|error| error.wrap_err(format!("in runtime module {}", self.id)))?;
      return Ok(
        source
          .transformed_source
          .expect("transformed custom runtime source"),
      );
    }
    Ok(source)
  }

  pub fn get_source_map_kind(&self) -> &SourceMapKind {
    &self.source_map_kind
  }

  pub fn set_source_map_kind(&mut self, source_map_kind: SourceMapKind) {
    self.source_map_kind = source_map_kind;
  }

  pub fn size(&self) -> f64 {
    self
      .cached_generated_code
      .get()
      .map_or(0f64, |cached_generated_code| {
        cached_generated_code.size() as f64
      })
  }
}

pub fn runtime_module_get_generated_code<'a>(
  module: &'a dyn RuntimeModule,
  common: &'a RuntimeModuleCommon,
  compilation: &'a Compilation,
) -> BoxFuture<'a, Result<Arc<dyn Source>>> {
  Box::pin(async move {
    let result: Result<&BoxSource> = common
      .cached_generated_code
      .get_or_try_init(|| async {
        let runtime_template = compilation
          .runtime_template
          .create_runtime_module_code_template();
        let context = RuntimeModuleGenerateContext {
          compilation,
          runtime_template: &runtime_template,
        };
        let source_str = module.generate_with_custom(&context).await?;
        let source_map_kind = module.get_source_map_kind();
        Ok(if source_map_kind.enabled() {
          OriginalSource::new(source_str, module.identifier().as_str()).boxed()
        } else {
          RawStringSource::from(source_str).boxed()
        })
      })
      .await;
    let source = result?.clone();
    Ok(source)
  })
}

pub async fn runtime_module_code_generation(
  module: &dyn RuntimeModule,
  common: &RuntimeModuleCommon,
  ctx: &mut ModuleCodeGenerationContext<'_>,
) -> Result<CodeGenerationResultBuilder> {
  let mut result = CodeGenerationResultBuilder::default();
  let source = runtime_module_get_generated_code(module, common, ctx.compilation).await?;
  result.add(SourceType::Runtime, source);
  Ok(result)
}

pub async fn runtime_module_get_runtime_hash(
  module: &dyn RuntimeModule,
  common: &RuntimeModuleCommon,
  compilation: &Compilation,
) -> Result<RspackHashDigest> {
  let mut hasher = RspackHasher::from(&compilation.options.output);
  module.name().hash(&mut hasher);
  module.stage().hash(&mut hasher);
  if module.full_hash() || module.dependent_hash() {
    let runtime_template = compilation
      .runtime_template
      .create_runtime_module_code_template();
    let context = RuntimeModuleGenerateContext {
      compilation,
      runtime_template: &runtime_template,
    };
    RspackHash::hash(&module.generate_with_custom(&context).await?, &mut hasher);
  } else {
    use std::hash::Hash;

    runtime_module_get_generated_code(module, common, compilation)
      .await?
      .hash(&mut hasher);
  }
  Ok(hasher.digest(&compilation.options.output.hash_digest))
}

#[async_trait]
pub trait RuntimeModule:
  Module + CustomSourceRuntimeModule + AttachableRuntimeModule + NamedRuntimeModule
{
  fn is_custom(&self) -> bool {
    false
  }

  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Normal
  }
  fn full_hash(&self) -> bool {
    false
  }
  fn dependent_hash(&self) -> bool {
    false
  }
  // if wrap iife
  fn should_isolate(&self, runtime_mode: RuntimeMode) -> bool {
    matches!(runtime_mode, RuntimeMode::Webpack)
  }
  fn template(&self) -> Vec<(String, String)> {
    vec![]
  }
  /// Names that this runtime module may declare in the surrounding chunk scope.
  ///
  /// Runtime modules backed by EJS templates should derive these names from top-level `var()` and
  /// `fn()` declarations. `#[impl_runtime_module]` registers the provider so the names can be
  /// reserved before runtime module instances are created.
  fn runtime_module_variables() -> &'static [&'static str]
  where
    Self: Sized;
  async fn generate(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String>;

  /// Records built-in state independently of codegen caches and custom source overrides.
  fn get_update_data(
    &self,
    _compilation: &Compilation,
  ) -> Option<Arc<dyn RuntimeModuleUpdateData>> {
    None
  }

  /// Generates code to migrate the previous runtime state during HMR apply.
  /// `None` uses the ordinary full source; `Some("")` explicitly emits no update code.
  /// This source uses the same scope and runtime requirements as `generate`, but
  /// must never replace the full source in codegen caches or runtime hashes.
  async fn generate_update(
    &self,
    _context: &RuntimeModuleGenerateContext<'_>,
    _previous: &dyn RuntimeModuleUpdateData,
  ) -> rspack_error::Result<Option<String>> {
    Ok(None)
  }

  fn runtime_requirements(&self, _compilation: &Compilation) -> RuntimeModuleRuntimeRequirements {
    Default::default()
  }
}

pub struct RuntimeModuleVariableProvider {
  pub variables: fn() -> &'static [&'static str],
}

inventory::collect!(RuntimeModuleVariableProvider);

pub fn all_runtime_module_variables() -> impl Iterator<Item = &'static str> {
  inventory::iter::<RuntimeModuleVariableProvider>
    .into_iter()
    .flat_map(|provider| (provider.variables)().iter().copied())
}

pub trait AttachableRuntimeModule {
  fn attach(&mut self, chunk: ChunkUkey);
}

pub trait NamedRuntimeModule {
  fn name(&self) -> Identifier;
}

#[async_trait]
pub trait CustomSourceRuntimeModule {
  /// Replace hook source, transforming and collecting its requirements atomically.
  fn set_custom_source(&mut self, source: String) -> Result<()>;
  /// Analyze the initial JS-generated source while preserving later hash regeneration.
  fn set_generated_source(&mut self, source: String) -> Result<()>;
  async fn generate_with_custom(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> Result<String>;
  fn get_custom_source(&self) -> &RuntimeModuleCustomSource;
  fn get_constructor_name(&self) -> String;
}

pub type BoxRuntimeModule = Box<dyn RuntimeModule>;

#[cacheable]
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum RuntimeModuleStage {
  #[default]
  Normal, // Runtime modules without any dependencies to other runtime modules
  Basic,   // Runtime modules with simple dependencies on other runtime modules
  Attach,  // Runtime modules which attach to handlers of other runtime modules
  Trigger, // Runtime modules which trigger actions on bootstrap
}

impl From<u32> for RuntimeModuleStage {
  fn from(stage: u32) -> Self {
    match stage {
      0 => RuntimeModuleStage::Normal,
      5 => RuntimeModuleStage::Basic,
      10 => RuntimeModuleStage::Attach,
      20 => RuntimeModuleStage::Trigger,
      _ => RuntimeModuleStage::Normal,
    }
  }
}

impl From<RuntimeModuleStage> for u32 {
  fn from(value: RuntimeModuleStage) -> Self {
    match value {
      RuntimeModuleStage::Normal => 0,
      RuntimeModuleStage::Basic => 5,
      RuntimeModuleStage::Attach => 10,
      RuntimeModuleStage::Trigger => 20,
    }
  }
}

impl RspackHash for RuntimeModuleStage {
  fn hash(&self, state: &mut RspackHasher) {
    let stage: u32 = self.clone().into();
    stage.hash(state);
  }
}

pub trait RuntimeModuleExt {
  fn boxed(self) -> Box<dyn RuntimeModule>;
}

impl<T: RuntimeModule + 'static> RuntimeModuleExt for T {
  fn boxed(self) -> Box<dyn RuntimeModule> {
    Box::new(self)
  }
}
