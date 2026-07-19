use quote::quote;
use syn::{ItemStruct, parse::Parser, parse_macro_input};

pub fn impl_runtime_module(
  _args: proc_macro::TokenStream,
  tokens: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
  let mut input = parse_macro_input!(tokens as ItemStruct);
  let name = &input.ident;
  let generics = &input.generics;
  let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
  let origin_fields = input.fields.clone();

  if let syn::Fields::Named(ref mut fields) = input.fields {
    fields.named.push(
      syn::Field::parse_named
        .parse2(quote! { pub common: ::rspack_core::RuntimeModuleCommon })
        .expect("Failed to parse new field for common"),
    );
  }

  let field_names = origin_fields
    .iter()
    .map(|field| field.ident.as_ref().expect("Expected named struct"))
    .collect::<Vec<_>>();
  let field_tys: Vec<&syn::Type> = origin_fields.iter().map(|field| &field.ty).collect();
  let with_default = quote! {
    #[allow(clippy::too_many_arguments)]
    fn with_default(runtime_template: &::rspack_core::RuntimeTemplate, #(#field_names: #field_tys,)*) -> Self {
      Self {
        common: ::rspack_core::RuntimeModuleCommon::with_default(runtime_template, stringify!(#name)),
        #(#field_names,)*
      }
    }
  };

  let with_name = quote! {
    #[allow(clippy::too_many_arguments)]
    fn with_name(runtime_template: &::rspack_core::RuntimeTemplate, name: &str, #(#field_names: #field_tys,)*) -> Self {
      Self {
        common: ::rspack_core::RuntimeModuleCommon::with_name(runtime_template, name),
        #(#field_names,)*
      }
    }
  };
  quote! {
    #[rspack_cacheable::cacheable]
    #input

    impl #impl_generics #name #ty_generics #where_clause {
      #with_default

      #with_name

      pub fn id(&self) -> &::rspack_collections::Identifier {
        self.common.id()
      }

      pub fn chunk(&self) -> Option<::rspack_core::ChunkUkey> {
        self.common.chunk()
      }

      async fn get_generated_code(
        &self,
        compilation: &::rspack_core::Compilation,
      ) -> ::rspack_error::Result<std::sync::Arc<dyn ::rspack_core::rspack_sources::Source>> {
        ::rspack_core::runtime_module_get_generated_code(self, &self.common, compilation).await
      }
    }

    impl #impl_generics ::rspack_core::CustomSourceRuntimeModule for #name #ty_generics #where_clause {
      fn set_custom_source(&mut self, source: String) -> () {
        self.common.set_custom_source(source);
      }
      fn get_custom_source(&self) -> Option<String> {
        self.common.get_custom_source()
      }
      fn get_constructor_name(&self) -> String {
        String::from(stringify!(#name))
      }
    }

    impl #impl_generics ::rspack_core::AttachableRuntimeModule for #name #ty_generics #where_clause {
      fn attach(&mut self, chunk: ::rspack_core::ChunkUkey) -> () {
        self.common.attach(chunk);
      }
    }

    impl #impl_generics ::rspack_core::NamedRuntimeModule for #name #ty_generics #where_clause {
      fn name(&self) -> ::rspack_collections::Identifier {
        *self.id()
      }
    }

    impl #impl_generics rspack_collections::Identifiable for #name #ty_generics #where_clause {
      fn identifier(&self) -> rspack_collections::Identifier {
        use rspack_core::NamedRuntimeModule;
        self.name()
      }
    }

    impl #impl_generics ::rspack_core::DependenciesBlock for #name #ty_generics #where_clause {
      fn add_block_id(&mut self, _: ::rspack_core::AsyncDependenciesBlockIdentifier) {
        unreachable!()
      }

      fn get_blocks(&self) -> &[::rspack_core::AsyncDependenciesBlockIdentifier] {
        unreachable!()
      }

      fn add_dependency_id(&mut self, _: ::rspack_core::DependencyId) {
        unreachable!()
      }

      fn remove_dependency_id(&mut self, _: ::rspack_core::DependencyId) {
        unreachable!()
      }

      fn get_dependencies(&self) -> &[::rspack_core::DependencyId] {
        unreachable!()
      }
    }

    #[rspack_cacheable::cacheable_dyn]
    #[async_trait::async_trait]
    impl #impl_generics ::rspack_core::Module for #name #ty_generics #where_clause {
      fn module_type(&self) -> &::rspack_core::ModuleType {
        &::rspack_core::ModuleType::Runtime
      }

      fn source_types(&self, module_graph: &::rspack_core::ModuleGraph) -> &[::rspack_core::SourceType] {
        &[::rspack_core::SourceType::JavaScript]
      }

      fn size(
        &self,
        _source_type: Option<&::rspack_core::SourceType>,
        _compilation: Option<&::rspack_core::Compilation>,
      ) -> f64 {
        self.common.size()
      }

      fn readable_identifier(&self, _context: &::rspack_core::Context) -> std::borrow::Cow<str> {
        use rspack_core::NamedRuntimeModule;
        self.name().as_str().into()
      }

      fn source(&self) -> Option<&::rspack_core::rspack_sources::BoxSource> {
        None
      }

      fn factory_meta(&self) -> Option<&::rspack_core::FactoryMeta> {
        None
      }

      fn set_factory_meta(&mut self, v: ::rspack_core::FactoryMeta) {}

      fn build_info(&self) -> &::rspack_core::BuildInfo {
        unreachable!()
      }

      fn build_info_mut(&mut self) -> &mut ::rspack_core::BuildInfo {
        unreachable!()
      }

      fn build_meta(&self) -> &::rspack_core::BuildMeta {
        unreachable!()
      }

      fn build_meta_mut(&mut self) -> &mut ::rspack_core::BuildMeta {
        unreachable!()
      }

      async fn code_generation(
        &self,
        code_generation_context: &mut ::rspack_core::ModuleCodeGenerationContext,
      ) -> rspack_error::Result<::rspack_core::CodeGenerationResult> {
        ::rspack_core::runtime_module_code_generation(self, &self.common, code_generation_context).await
      }

      async fn get_runtime_hash(
        &self,
        compilation: &::rspack_core::Compilation,
        runtime: Option<&::rspack_core::RuntimeSpec>,
      ) -> rspack_error::Result<::rspack_hash::RspackHashDigest> {
        ::rspack_core::runtime_module_get_runtime_hash(self, &self.common, compilation, runtime).await
      }

      async fn build(
        self: Box<Self>,
        _build_context: ::rspack_core::BuildContext,
        _compilation: Option<&::rspack_core::Compilation>,
      ) -> ::rspack_error::Result<::rspack_core::BuildResult> {
        Ok(::rspack_core::BuildResult {
          module: ::rspack_core::BoxModule::new(self),
          dependencies: vec![],
          blocks: vec![],
          optimization_bailouts: vec![],
        })
      }
    }

    impl #impl_generics rspack_error::Diagnosable for #name  #ty_generics #where_clause {
      fn add_diagnostic(&mut self, _diagnostic: rspack_error::Diagnostic) {
        unreachable!()
      }
      fn add_diagnostics(&mut self, _diagnostics: Vec<rspack_error::Diagnostic>) {
        unreachable!()
      }
      fn diagnostics(&self) -> std::borrow::Cow<'_, [rspack_error::Diagnostic]> {
        std::borrow::Cow::Owned(vec![])
      }
    }

    impl #impl_generics ::rspack_util::source_map::ModuleSourceMapConfig for #name #ty_generics #where_clause {
      fn get_source_map_kind(&self) -> &::rspack_util::source_map::SourceMapKind {
        self.common.get_source_map_kind()
      }

      fn set_source_map_kind(&mut self, source_map_kind: ::rspack_util::source_map::SourceMapKind) {
        self.common.set_source_map_kind(source_map_kind);
      }
    }
  }
  .into()
}
