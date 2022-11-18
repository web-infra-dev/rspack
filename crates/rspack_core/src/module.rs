use std::hash::Hash;
use std::{any::Any, borrow::Cow, fmt::Debug};

use async_trait::async_trait;

use rspack_error::{Error, Result, TWithDiagnosticArray};
use rspack_loader_runner::Loader;
use rspack_sources::Source;

use crate::{
  CodeGenerationResult, Compilation, CompilationContext, CompilerContext, CompilerOptions, Context,
  LoaderRunnerRunner, ModuleDependency, ModuleType, NormalModule, RawModule, SourceType,
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

pub trait DynHash {
  fn dyn_hash(&self, state: &mut dyn std::hash::Hasher);
}

impl<T: Hash> DynHash for T {
  fn dyn_hash(&self, mut state: &mut dyn std::hash::Hasher) {
    self.hash(&mut state);
  }
}

pub trait DynEq {
  fn dyn_eq(&self, other: &dyn Any) -> bool;
}

impl<T: Eq + Any> DynEq for T {
  fn dyn_eq(&self, other: &dyn Any) -> bool {
    if let Some(module) = other.downcast_ref::<T>() {
      self == module
    } else {
      false
    }
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
pub trait Module: Debug + Send + Sync + AsAny + DynHash + DynEq {
  fn module_type(&self) -> &ModuleType;

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

  fn update_hash(&self, state: &mut dyn std::hash::Hasher) {
    self.dyn_hash(state);
  }
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

#[async_trait::async_trait]
impl Module for Box<dyn Module> {
  fn module_type(&self) -> &ModuleType {
    self.as_ref().module_type()
  }

  fn source_types(&self) -> &[SourceType] {
    self.as_ref().source_types()
  }

  fn original_source(&self) -> Option<&dyn Source> {
    self.as_ref().original_source()
  }

  fn identifier(&self) -> Cow<str> {
    self.as_ref().identifier()
  }

  fn readable_identifier(&self, context: &Context) -> Cow<str> {
    self.as_ref().readable_identifier(context)
  }

  fn size(&self, source_type: &SourceType) -> f64 {
    self.as_ref().size(source_type)
  }

  async fn build(
    &mut self,
    build_context: BuildContext<'_>,
  ) -> Result<TWithDiagnosticArray<BuildResult>> {
    self.as_mut().build(build_context).await
  }

  fn code_generation(&self, compilation: &Compilation) -> Result<CodeGenerationResult> {
    self.as_ref().code_generation(compilation)
  }
}

impl PartialEq for dyn Module {
  fn eq(&self, other: &Self) -> bool {
    self.dyn_eq(other.as_any())
  }
}

impl Eq for dyn Module {}

impl Hash for dyn Module {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.dyn_hash(state)
  }
}

impl dyn Module + '_ {
  pub fn downcast_ref<T: Module + Any>(&self) -> Option<&T> {
    self.as_any().downcast_ref::<T>()
  }

  pub fn downcast_mut<T: Module + Any>(&mut self) -> Option<&mut T> {
    self.as_any_mut().downcast_mut::<T>()
  }
}

macro_rules! impl_module_downcast_helpers {
  ($ty:ty, $ident:ident) => {
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
impl_module_downcast_helpers!(RawModule, raw_module);

#[cfg(test)]
mod test {
  use std::borrow::Cow;
  use std::hash::{Hash, Hasher};

  use super::Module;
  use crate::{
    BuildContext, BuildResult, CodeGenerationResult, Compilation, Context, ModuleExt, ModuleType,
    SourceType,
  };

  use rspack_error::{Result, TWithDiagnosticArray};
  use rspack_sources::Source;

  #[derive(Debug, Eq)]
  struct RawModule(&'static str);

  impl PartialEq for RawModule {
    fn eq(&self, other: &Self) -> bool {
      self.identifier() == other.identifier()
    }
  }

  #[derive(Debug, Eq)]
  struct ExternalModule(&'static str);

  impl PartialEq for ExternalModule {
    fn eq(&self, other: &Self) -> bool {
      self.identifier() == other.identifier()
    }
  }

  impl Hash for RawModule {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
      self.identifier().hash(state);
    }
  }

  impl Hash for ExternalModule {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
      self.identifier().hash(state);
    }
  }

  macro_rules! impl_noop_trait_module_type {
    ($ident: ident) => {
      #[::async_trait::async_trait]
      impl Module for $ident {
        fn module_type(&self) -> &ModuleType {
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
          (stringify!($ident).to_owned() + &self.0).into()
        }

        fn readable_identifier(&self, _context: &Context) -> Cow<str> {
          (stringify!($ident).to_owned() + &self.0).into()
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
    let a: Box<dyn Module> = ExternalModule("a").boxed();
    let b: Box<dyn Module> = RawModule("a").boxed();

    assert!(a.downcast_ref::<ExternalModule>().is_some());
    assert!(b.downcast_ref::<RawModule>().is_some());

    let a = a.as_ref();
    let b = b.as_ref();
    assert!(a.downcast_ref::<ExternalModule>().is_some());
    assert!(b.downcast_ref::<RawModule>().is_some());
  }

  #[test]
  fn hash_should_work() {
    let e1: Box<dyn Module> = ExternalModule("e").boxed();
    let e2: Box<dyn Module> = ExternalModule("e").boxed();

    let mut state1 = xxhash_rust::xxh3::Xxh3::default();
    let mut state2 = xxhash_rust::xxh3::Xxh3::default();
    e1.hash(&mut state1);
    e2.hash(&mut state2);

    let hash1 = format!("{:x}", state1.finish());
    let hash2 = format!("{:x}", state2.finish());
    assert_eq!(hash1, hash2);

    let e3: Box<dyn Module> = ExternalModule("e3").boxed();
    let mut state3 = xxhash_rust::xxh3::Xxh3::default();
    e3.hash(&mut state3);

    let hash3 = format!("{:x}", state3.finish());
    assert_ne!(hash1, hash3);
  }
}
