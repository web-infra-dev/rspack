use quote::quote;
use syn::{parse::Parser, parse_macro_input, ItemStruct};

#[proc_macro_attribute]
pub fn impl_source_map_config_internal(
  _args: proc_macro::TokenStream,
  tokens: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
  let mut input = parse_macro_input!(tokens as ItemStruct);
  let name = &input.ident;

  if let syn::Fields::Named(ref mut fields) = input.fields {
    fields.named.push(
      syn::Field::parse_named
        .parse2(quote! { pub source_map_option: crate::module::SourceMapKind })
        .unwrap(),
    );
  }

  quote! {
    #input

    impl crate::module::SourceMapGenConfig for #name {
      fn get_source_map_kind(&self) -> &crate::module::SourceMapKind {
        &self.source_map_option
      }

      fn set_source_map_kind(&mut self, source_map_option: crate::module::SourceMapKind) {
        self.source_map_option = source_map_option;
      }
    }
  }
  .into()
}

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
        .parse2(quote! { pub source_map_option: ::rspack_core::module::SourceMapKind })
        .unwrap(),
    );
  }

  quote! {
    #input

    impl ::rspack_core::module::SourceMapGenConfig for #name {
      fn get_source_map_kind(&self) -> &::rspack_core::module::SourceMapKind {
        &self.source_map_option
      }

      fn set_source_map_kind(&mut self, source_map_option: ::rspack_core::module::SourceMapKind) {
        self.source_map_option = source_map_option;
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
        .parse2(quote! { pub source_map_option: ::rspack_core::module::SourceMapKind })
        .unwrap(),
    );
  }

  quote! {
    #input

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

      fn code_generation(
        &self,
        compilation: &::rspack_core::Compilation,
        _runtime: Option<&::rspack_core::RuntimeSpec>,
      ) -> rspack_error::Result<::rspack_core::CodeGenerationResult> {
        let mut result = ::rspack_core::CodeGenerationResult::default();
        result.add(::rspack_core::SourceType::JavaScript, self.generate(compilation));
        result.set_hash(
          &compilation.options.output.hash_function,
          &compilation.options.output.hash_digest,
          &compilation.options.output.hash_salt,
        );
        Ok(result)
      }
    }

    impl rspack_error::Diagnosable for #name {}

    impl ::rspack_core::module::SourceMapGenConfig for #name {
      fn get_source_map_kind(&self) -> &::rspack_core::module::SourceMapKind {
        &self.source_map_option
      }

      fn set_source_map_kind(&mut self, source_map_option: ::rspack_core::module::SourceMapKind) {
        self.source_map_option = source_map_option;
      }
    }
  }
  .into()
}
