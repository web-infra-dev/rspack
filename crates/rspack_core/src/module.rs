use std::{any::Any, fmt::Debug};

use async_trait::async_trait;

use rspack_error::{Error, Result, TWithDiagnosticArray};
use rspack_loader_runner::Loader;
use rspack_sources::Source;

use crate::{
  CodeGenerationResult, Compilation, CompilationContext, CompilerContext, CompilerOptions, Context,
  LoaderRunnerRunner, ModuleDependency, ModuleType, NormalModule, SourceType,
};

pub trait AsAny {
  fn as_any(&self) -> &dyn Any;
  fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Any> AsAny for T {
  fn as_any(&self) -> &dyn Any {
    self
  }
  fn as_any_mut(&mut self) -> &mut dyn Any {
    self
  }
}

pub struct BuildContext<'a> {
  pub loader_runner_runner: &'a LoaderRunnerRunner,
  pub resolved_loaders: Vec<&'a dyn Loader<CompilerContext, CompilationContext>>,
  pub compiler_options: &'a CompilerOptions,
}

#[derive(Debug, Default)]
pub struct BuildResult {
  pub dependencies: Vec<ModuleDependency>,
}

#[async_trait]
pub trait Module: Debug + Send + Sync + AsAny {
  fn module_type(&self) -> ModuleType;

  fn source_types(&self) -> &[SourceType];

  fn original_source(&self) -> Option<&dyn Source>;

  fn identifier(&self) -> String;

  fn readable_identifier(&self, _context: &Context) -> String;

  fn size(&self, _source_type: &SourceType) -> f64;

  async fn build(
    &mut self,
    _build_context: BuildContext<'_>,
  ) -> Result<TWithDiagnosticArray<BuildResult>>;

  fn code_generation(&self, _compilation: &Compilation) -> Result<CodeGenerationResult>;
}

pub trait ModuleExt {
  fn boxed(self) -> Box<dyn Module>;
}

impl<T: Module + 'static> ModuleExt for T {
  fn boxed(self) -> Box<dyn Module> {
    Box::new(self)
  }
}

pub type BoxModule = Box<dyn Module>;

impl dyn Module + '_ {
  pub fn downcast_ref<T: Module + Any>(&self) -> Option<&T> {
    self.as_any().downcast_ref::<T>()
  }

  pub fn downcast_mut<T: Module + Any>(&mut self) -> Option<&mut T> {
    self.as_any_mut().downcast_mut::<T>()
  }

  pub fn as_normal_module(&self) -> Option<&NormalModule> {
    self.as_any().downcast_ref::<NormalModule>()
  }

  pub fn as_normal_module_mut(&mut self) -> Option<&mut NormalModule> {
    self.as_any().downcast_mut::<NormalModule>()
  }

  pub fn try_as_normal_module(&self) -> Result<&NormalModule> {
    self.as_normal_module().ok_or_else(|| {
      Error::InternalError(format!(
        "Failed to cast module {} to a NormalModule",
        self.identifier()
      ))
    })
  }

  pub fn try_as_normal_module_mut(&mut self) -> Result<&mut NormalModule> {
    self.as_normal_module_mut().ok_or_else(|| {
      Error::InternalError(format!(
        "Failed to cast module {} to a NormalModule",
        self.identifier()
      ))
    })
  }
}

#[cfg(test)]
mod test {
  use super::Module;
  use crate::{
    BuildContext, BuildResult, CodeGenerationResult, Compilation, Context, ModuleExt, ModuleType,
    SourceType,
  };

  use rspack_error::{Result, TWithDiagnosticArray};
  use rspack_sources::Source;

  #[derive(Debug)]
  struct RawModule {}

  #[derive(Debug)]
  struct ExternalModule {}

  #[async_trait::async_trait]
  impl Module for RawModule {
    fn module_type(&self) -> ModuleType {
      unreachable!()
    }

    fn source_types(&self) -> &[SourceType] {
      unreachable!()
    }

    fn original_source(&self) -> Option<&dyn Source> {
      unreachable!()
    }

    fn size(&self, _source_type: &SourceType) -> f64 {
      unreachable!()
    }

    fn identifier(&self) -> String {
      "raw".to_owned()
    }

    fn readable_identifier(&self, _context: &Context) -> String {
      unreachable!()
    }

    async fn build(
      &mut self,
      _build_context: BuildContext<'_>,
    ) -> Result<TWithDiagnosticArray<BuildResult>> {
      unreachable!()
    }

    fn code_generation(&self, _compilation: &Compilation) -> Result<CodeGenerationResult> {
      unreachable!()
    }
  }

  #[async_trait::async_trait]
  impl Module for ExternalModule {
    fn module_type(&self) -> ModuleType {
      unreachable!()
    }

    fn source_types(&self) -> &[SourceType] {
      unreachable!()
    }

    fn original_source(&self) -> Option<&dyn Source> {
      unreachable!()
    }

    fn size(&self, _source_type: &SourceType) -> f64 {
      unreachable!()
    }

    fn identifier(&self) -> String {
      "external".to_owned()
    }

    fn readable_identifier(&self, _context: &Context) -> String {
      unreachable!()
    }

    async fn build(
      &mut self,
      _build_context: BuildContext<'_>,
    ) -> Result<TWithDiagnosticArray<BuildResult>> {
      unreachable!()
    }

    fn code_generation(&self, _compilation: &Compilation) -> Result<CodeGenerationResult> {
      unreachable!()
    }
  }

  #[test]
  fn should_downcast_successfully() {
    let a: Box<dyn Module> = ExternalModule {}.boxed();
    let b: Box<dyn Module> = RawModule {}.boxed();

    assert!(a.downcast_ref::<ExternalModule>().is_some());
    assert!(b.downcast_ref::<RawModule>().is_some());

    let a = a.as_ref();
    let b = b.as_ref();
    assert!(a.downcast_ref::<ExternalModule>().is_some());
    assert!(b.downcast_ref::<RawModule>().is_some());
  }
}
