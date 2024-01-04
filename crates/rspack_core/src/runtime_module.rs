use rspack_identifier::Identifier;
use rspack_sources::BoxSource;

use crate::{ChunkUkey, Compilation, Module};

pub trait RuntimeModule: Module {
  fn name(&self) -> Identifier;
  fn generate(&self, compilation: &Compilation) -> BoxSource;
  fn attach(&mut self, _chunk: ChunkUkey) {}
  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Normal
  }
  // webpack fullHash || dependentHash
  fn cacheable(&self) -> bool {
    true
  }
  // if wrap iife
  fn should_isolate(&self) -> bool {
    false
  }
}

pub type BoxRuntimeModule = Box<dyn RuntimeModule>;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RuntimeModuleStage {
  Normal,  // Runtime modules without any dependencies to other runtime modules
  Basic,   // Runtime modules with simple dependencies on other runtime modules
  Attach,  // Runtime modules which attach to handlers of other runtime modules
  Trigger, // Runtime modules which trigger actions on bootstrap
}

pub trait RuntimeModuleExt {
  fn boxed(self) -> Box<dyn RuntimeModule>;
}

impl<T: RuntimeModule + 'static> RuntimeModuleExt for T {
  fn boxed(self) -> Box<dyn RuntimeModule> {
    Box::new(self)
  }
}

#[macro_export]
macro_rules! impl_runtime_module {
  ($ident:ident) => {
    use rspack_error::Diagnostic;
    impl rspack_identifier::Identifiable for $ident {
      fn identifier(&self) -> rspack_identifier::Identifier {
        self.name()
      }
    }

    impl PartialEq for $ident {
      fn eq(&self, other: &Self) -> bool {
        self.name() == other.name()
      }
    }

    impl std::hash::Hash for $ident {
      fn hash<H: std::hash::Hasher>(&self, _state: &mut H) {
        unreachable!()
      }
    }

    impl $crate::DependenciesBlock for $ident {
      fn add_block_id(&mut self, _: $crate::AsyncDependenciesBlockIdentifier) {
        unreachable!()
      }

      fn get_blocks(&self) -> &[$crate::AsyncDependenciesBlockIdentifier] {
        unreachable!()
      }

      fn add_dependency_id(&mut self, _: $crate::DependencyId) {
        unreachable!()
      }

      fn get_dependencies(&self) -> &[$crate::DependencyId] {
        unreachable!()
      }
    }

    impl $crate::Module for $ident {
      fn module_type(&self) -> &$crate::ModuleType {
        &$crate::ModuleType::Runtime
      }

      fn source_types(&self) -> &[$crate::SourceType] {
        &[$crate::SourceType::JavaScript]
      }

      fn size(&self, _source_type: &$crate::SourceType) -> f64 {
        // TODO
        160.0
      }

      fn get_diagnostics(&self) -> Vec<Diagnostic> {
        vec![]
      }

      fn readable_identifier(&self, _context: &$crate::Context) -> std::borrow::Cow<str> {
        self.name().as_str().into()
      }

      fn original_source(&self) -> Option<&dyn $crate::rspack_sources::Source> {
        None
      }

      fn build_info(&self) -> Option<&$crate::BuildInfo> {
        None
      }

      fn build_meta(&self) -> Option<&$crate::BuildMeta> {
        None
      }

      fn set_module_build_info_and_meta(
        &mut self,
        build_info: $crate::BuildInfo,
        build_meta: $crate::BuildMeta,
      ) {
      }

      fn code_generation(
        &self,
        compilation: &$crate::Compilation,
        _runtime: Option<&$crate::RuntimeSpec>,
      ) -> rspack_error::Result<$crate::CodeGenerationResult> {
        let mut result = $crate::CodeGenerationResult::default();
        result.add($crate::SourceType::JavaScript, self.generate(compilation));
        result.set_hash(
          &compilation.options.output.hash_function,
          &compilation.options.output.hash_digest,
          &compilation.options.output.hash_salt,
        );
        Ok(result)
      }
    }

    impl rspack_error::Diagnosable for $ident {}
  };
}
