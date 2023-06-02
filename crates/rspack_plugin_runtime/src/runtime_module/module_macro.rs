#[macro_export]
macro_rules! impl_runtime_module {
  ($ident: ident) => {
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

    impl rspack_core::Module for $ident {
      fn module_type(&self) -> &rspack_core::ModuleType {
        &rspack_core::ModuleType::Js
      }

      fn source_types(&self) -> &[rspack_core::SourceType] {
        &[rspack_core::SourceType::JavaScript]
      }

      fn size(&self, _source_type: &rspack_core::SourceType) -> f64 {
        // TODO
        160.0
      }

      fn readable_identifier(&self, _context: &rspack_core::Context) -> std::borrow::Cow<str> {
        self.name().as_str().into()
      }

      fn original_source(&self) -> Option<&dyn rspack_core::rspack_sources::Source> {
        None
      }

      fn code_generation(
        &self,
        compilation: &rspack_core::Compilation,
      ) -> rspack_error::Result<rspack_core::CodeGenerationResult> {
        let mut result = rspack_core::CodeGenerationResult::default();
        result.add(
          rspack_core::SourceType::JavaScript,
          rspack_core::GenerationResult::from(rspack_core::AstOrSource::from(
            self.generate(compilation),
          )),
        );
        result.set_hash(
          &compilation.options.output.hash_function,
          &compilation.options.output.hash_digest,
          &compilation.options.output.hash_salt,
        );
        Ok(result)
      }
    }
  };
}
