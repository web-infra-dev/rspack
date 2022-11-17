use std::{any::Any, borrow::Cow, fmt::Debug};

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

  fn identifier(&self) -> Cow<str>;

  fn readable_identifier(&self, _context: &Context) -> Cow<str>;

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
}

macro_rules! impl_module_downcast_helpers {
  ($ty:ty, $ident: ident) => {
    impl dyn Module + '_ {
      ::paste::paste! {
        pub fn [<as_ $ident>](&self) -> Option<& $ty> {
          self.as_any().downcast_ref::<$ty>()
        }

        pub fn [<as_ $ident _mut>](&mut self) -> Option<&mut $ty> {
          self.as_any_mut().downcast_mut::<$ty>()
        }

        pub fn [<try_as_ $ident>](&self) -> Result<& $ty> {
          self.[<as_ $ident>]().ok_or_else(|| {
            Error::InternalError(format!(
              "Failed to cast module to a {}",
              stringify!($ty)
            ))
          })
        }

        pub fn [<try_as_ $ident _mut>](&mut self) -> Result<&mut $ty> {
          self.[<as_ $ident _mut>]().ok_or_else(|| {
            Error::InternalError(format!(
              "Failed to cast module to a {}",
              stringify!($ty)
            ))
          })
        }
      }
    }
  };
}

impl_module_downcast_helpers!(NormalModule, normal_module);

#[cfg(test)]
mod test {
  use std::borrow::Cow;

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

  macro_rules! impl_noop_trait_module_type {
    ($ident: ident) => {
      #[::async_trait::async_trait]
      impl Module for $ident {
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

        fn identifier(&self) -> Cow<str> {
          stringify!($ident).into()
        }

        fn readable_identifier(&self, _context: &Context) -> Cow<str> {
          stringify!($ident).into()
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
    };
  }

  impl_noop_trait_module_type!(RawModule);
  impl_noop_trait_module_type!(ExternalModule);

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
