use quote::quote;
use syn::{parse::Parser, parse_macro_input, ItemStruct};

#[proc_macro_attribute]
pub fn impl_source_map_config(
  _args: proc_macro::TokenStream,
  tokens: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
  let mut input = parse_macro_input!(tokens as ItemStruct);
  let name = &input.ident;

  if let syn::Fields::Named(ref mut fields) = input.fields {
    fields.named.push(
      syn::Field::parse_named
        .parse2(quote! { pub source_map_kind: ::rspack_util::source_map::SourceMapKind })
        .expect("Failed to parse new field for source_map_kind"),
    );
  }

  quote! {
    #input

    impl ::rspack_util::source_map::ModuleSourceMapConfig for #name {
      fn get_source_map_kind(&self) -> &::rspack_util::source_map::SourceMapKind {
        &self.source_map_kind
      }

      fn set_source_map_kind(&mut self, source_map_kind: ::rspack_util::source_map::SourceMapKind) {
        self.source_map_kind = source_map_kind;
      }
    }
  }
  .into()
}

#[proc_macro_attribute]
pub fn impl_runtime_module(
  _args: proc_macro::TokenStream,
  tokens: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
  let mut input = parse_macro_input!(tokens as ItemStruct);
  let name = &input.ident;

  if let syn::Fields::Named(ref mut fields) = input.fields {
    fields.named.push(
      syn::Field::parse_named
        .parse2(quote! { pub source_map_kind: ::rspack_util::source_map::SourceMapKind })
        .expect("Failed to parse new field for source_map_kind"),
    );
    fields.named.push(
      syn::Field::parse_named
        .parse2(
          quote! { pub custom_source: Option<::std::sync::Arc<::rspack_core::rspack_sources::OriginalSource>> },
        )
        .expect("Failed to parse new field for original_source"),
    );
  }

  quote! {
    #input

    impl ::rspack_core::CustomSourceRuntimeModule for #name {
      fn set_custom_source(&mut self, source: ::rspack_core::rspack_sources::OriginalSource) -> () {
        self.custom_source = Some(::std::sync::Arc::new(source));
      }
      fn get_custom_source(&self) -> Option<::std::sync::Arc<::rspack_core::rspack_sources::OriginalSource>> {
        self.custom_source.clone()
      }
      fn get_constructor_name(&self) -> String {
        String::from(stringify!(#name))
      }
    }

    impl rspack_identifier::Identifiable for #name {
      fn identifier(&self) -> rspack_identifier::Identifier {
        self.name()
      }
    }

    impl PartialEq for #name {
      fn eq(&self, other: &Self) -> bool {
        self.name() == other.name()
      }
    }

    impl std::hash::Hash for #name {
      fn hash<H: std::hash::Hasher>(&self, _state: &mut H) {
        unreachable!()
      }
    }

    impl ::rspack_core::DependenciesBlock for #name {
      fn add_block_id(&mut self, _: ::rspack_core::AsyncDependenciesBlockIdentifier) {
        unreachable!()
      }

      fn get_blocks(&self) -> &[::rspack_core::AsyncDependenciesBlockIdentifier] {
        unreachable!()
      }

      fn add_dependency_id(&mut self, _: ::rspack_core::DependencyId) {
        unreachable!()
      }

      fn get_dependencies(&self) -> &[::rspack_core::DependencyId] {
        unreachable!()
      }
    }

    impl ::rspack_core::Module for #name {
      fn module_type(&self) -> &::rspack_core::ModuleType {
        &::rspack_core::ModuleType::Runtime
      }

      fn source_types(&self) -> &[::rspack_core::SourceType] {
        &[::rspack_core::SourceType::JavaScript]
      }

      fn size(&self, _source_type: &::rspack_core::SourceType) -> f64 {
        // TODO
        160.0
      }

      fn readable_identifier(&self, _context: &::rspack_core::Context) -> std::borrow::Cow<str> {
        self.name().as_str().into()
      }

      fn original_source(&self) -> Option<&dyn ::rspack_core::rspack_sources::Source> {
        None
      }

      fn build_info(&self) -> Option<&::rspack_core::BuildInfo> {
        None
      }

      fn build_meta(&self) -> Option<&::rspack_core::BuildMeta> {
        None
      }

      fn set_module_build_info_and_meta(
        &mut self,
        build_info: ::rspack_core::BuildInfo,
        build_meta: ::rspack_core::BuildMeta,
      ) {
      }

      fn get_diagnostics(&self) -> Vec<::rspack_error::Diagnostic> {
        vec![]
      }

      fn code_generation(
        &self,
        compilation: &::rspack_core::Compilation,
        _runtime: Option<&::rspack_core::RuntimeSpec>,
        _: Option<::rspack_core::ConcatenationScope>,
      ) -> rspack_error::Result<::rspack_core::CodeGenerationResult> {
        let mut result = ::rspack_core::CodeGenerationResult::default();
        result.add(::rspack_core::SourceType::JavaScript, self.generate_with_custom(compilation));
        result.set_hash(
          &compilation.options.output.hash_function,
          &compilation.options.output.hash_digest,
          &compilation.options.output.hash_salt,
        );
        Ok(result)
      }
    }

    impl rspack_error::Diagnosable for #name {}

    impl ::rspack_util::source_map::ModuleSourceMapConfig for #name {
      fn get_source_map_kind(&self) -> &::rspack_util::source_map::SourceMapKind {
        &self.source_map_kind
      }

      fn set_source_map_kind(&mut self, source_map_kind: ::rspack_util::source_map::SourceMapKind) {
        self.source_map_kind = source_map_kind;
      }
    }
  }
  .into()
}
