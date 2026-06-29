use std::{fmt::Debug, sync::Arc};

use async_trait::async_trait;
use rspack_cacheable::cacheable;
use rspack_collections::Identifier;
use rspack_error::Result;
use rspack_hash::RspackHashDigest;
use rspack_sources::{BoxSource, OriginalSource, RawStringSource, Source, SourceExt};
use rspack_util::{ext::DynHash, source_map::SourceMapKind};
use tokio::sync::OnceCell;

use crate::{
  ChunkUkey, CodeGenerationResult, Compilation, Module, ModuleCodeGenerationContext,
  RuntimeCodeTemplate, RuntimeGlobals, RuntimeSpec, RuntimeTemplate, SourceType,
};

pub struct RuntimeModuleGenerateContext<'a> {
  pub compilation: &'a Compilation,
  pub runtime_template: &'a RuntimeCodeTemplate<'a>,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct RuntimeModuleRuntimeRequirements {
  pub dependencies: RuntimeGlobals,
  pub weak: RuntimeGlobals,
  pub write: RuntimeGlobals,
  pub force_context: RuntimeGlobals,
}

impl RuntimeModuleRuntimeRequirements {
  pub fn lexical_requirements(&self) -> RuntimeGlobals {
    self.dependencies | self.weak | self.write | self.force_context
  }
}

#[cacheable]
#[derive(Debug, Default, Clone)]
pub struct RuntimeModuleCommon {
  pub id: Identifier,
  pub chunk: Option<ChunkUkey>,
  pub source_map_kind: SourceMapKind,
  pub custom_source: Option<String>,
  #[cacheable(with=rspack_cacheable::with::Skip)]
  pub cached_generated_code: Arc<OnceCell<BoxSource>>,
}

impl RuntimeModuleCommon {
  pub fn with_default(runtime_template: &RuntimeTemplate, type_name: &str) -> Self {
    Self {
      source_map_kind: SourceMapKind::empty(),
      custom_source: None,
      cached_generated_code: Default::default(),
      chunk: None,
      id: runtime_template.create_runtime_module_identifier(type_name),
    }
  }

  pub fn with_name(runtime_template: &RuntimeTemplate, name: &str) -> Self {
    Self {
      source_map_kind: SourceMapKind::empty(),
      custom_source: None,
      cached_generated_code: Default::default(),
      chunk: None,
      id: runtime_template.create_custom_runtime_module_identifier(name),
    }
  }

  pub fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }

  pub fn name(&self) -> Identifier {
    self.id
  }

  pub fn id(&self) -> &Identifier {
    &self.id
  }

  pub fn chunk(&self) -> Option<ChunkUkey> {
    self.chunk
  }

  pub fn set_custom_source(&mut self, source: String) {
    self.custom_source = Some(source);
  }

  pub fn get_custom_source(&self) -> Option<String> {
    self.custom_source.clone()
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

pub async fn runtime_module_get_generated_code(
  module: &dyn RuntimeModule,
  common: &RuntimeModuleCommon,
  compilation: &Compilation,
) -> Result<Arc<dyn Source>> {
  let result: Result<&BoxSource> = common
    .cached_generated_code
    .get_or_try_init(|| async {
      let runtime_template = compilation.runtime_template.create_runtime_code_template();
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
}

pub async fn runtime_module_code_generation(
  module: &dyn RuntimeModule,
  common: &RuntimeModuleCommon,
  ctx: &mut ModuleCodeGenerationContext<'_>,
) -> Result<CodeGenerationResult> {
  let mut result = CodeGenerationResult::default();
  let source = runtime_module_get_generated_code(module, common, ctx.compilation).await?;
  result.add(SourceType::Runtime, source);
  Ok(result)
}

pub async fn runtime_module_get_runtime_hash(
  module: &dyn RuntimeModule,
  common: &RuntimeModuleCommon,
  compilation: &Compilation,
  _runtime: Option<&RuntimeSpec>,
) -> Result<RspackHashDigest> {
  let mut hasher = rspack_hash::RspackHash::from(&compilation.options.output);
  module.name().dyn_hash(&mut hasher);
  module.stage().dyn_hash(&mut hasher);
  if module.full_hash() || module.dependent_hash() {
    use std::hash::Hash;

    let runtime_template = compilation.runtime_template.create_runtime_code_template();
    let context = RuntimeModuleGenerateContext {
      compilation,
      runtime_template: &runtime_template,
    };
    module
      .generate_with_custom(&context)
      .await?
      .hash(&mut hasher);
  } else {
    runtime_module_get_generated_code(module, common, compilation)
      .await?
      .dyn_hash(&mut hasher);
  }
  Ok(hasher.digest(&compilation.options.output.hash_digest))
}

#[async_trait]
pub trait RuntimeModule:
  Module + CustomSourceRuntimeModule + AttachableRuntimeModule + NamedRuntimeModule
{
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
  fn should_isolate(&self) -> bool {
    true
  }
  fn template(&self) -> Vec<(String, String)> {
    vec![]
  }
  async fn generate(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String>;
  async fn generate_with_custom(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    if let Some(custom_source) = self.get_custom_source() {
      Ok(custom_source)
    } else {
      self.generate(context).await
    }
  }
  fn runtime_requirements(&self, _compilation: &Compilation) -> RuntimeModuleRuntimeRequirements {
    Default::default()
  }
}

pub trait AttachableRuntimeModule {
  fn attach(&mut self, chunk: ChunkUkey);
}

pub trait NamedRuntimeModule {
  fn name(&self) -> Identifier;
}

pub trait CustomSourceRuntimeModule {
  fn set_custom_source(&mut self, source: String);
  fn get_custom_source(&self) -> Option<String>;
  fn get_constructor_name(&self) -> String;
}

pub type BoxRuntimeModule = Box<dyn RuntimeModule>;

#[cacheable]
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

pub trait RuntimeModuleExt {
  fn boxed(self) -> Box<dyn RuntimeModule>;
}

impl<T: RuntimeModule + 'static> RuntimeModuleExt for T {
  fn boxed(self) -> Box<dyn RuntimeModule> {
    Box::new(self)
  }
}
