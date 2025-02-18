use quote::quote;
use syn::{parse::Parser, parse_macro_input, ItemStruct};

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
        .parse2(quote! { pub source_map_kind: ::rspack_util::source_map::SourceMapKind })
        .expect("Failed to parse new field for source_map_kind"),
    );
    fields.named.push(
      syn::Field::parse_named
        .parse2(quote! {
            #[cacheable(with=rspack_cacheable::with::AsOption<rspack_cacheable::with::AsPreset>)]
            pub custom_source: Option<::rspack_core::rspack_sources::BoxSource>
        })
        .expect("Failed to parse new field for custom_source"),
    );
    fields.named.push(
      syn::Field::parse_named
            .parse2(quote! {
                #[cacheable(with=rspack_cacheable::with::Skip)]
                pub cached_generated_code: std::sync::RwLock<Option<::rspack_core::rspack_sources::BoxSource>>
            })
        .expect("Failed to parse new field for cached_generated_code"),
    );
  }

  let field_names = origin_fields
    .iter()
    .map(|field| field.ident.as_ref().expect("Expected named struct"))
    .collect::<Vec<_>>();
  let field_tys: Vec<&syn::Type> = origin_fields.iter().map(|field| &field.ty).collect();
  let with_default = quote! {
    #[allow(clippy::too_many_arguments)]
    fn with_default(#(#field_names: #field_tys,)*) -> Self {
      Self {
        source_map_kind: ::rspack_util::source_map::SourceMapKind::empty(),
        custom_source: None,
        cached_generated_code: Default::default(),
        #(#field_names,)*
      }
    }
  };

  quote! {
    #[rspack_cacheable::cacheable]
    #input

    impl #impl_generics #name #ty_generics #where_clause {
      #with_default

      fn get_generated_code(
        &self,
        compilation: &::rspack_core::Compilation,
      ) -> ::rspack_error::Result<std::sync::Arc<dyn ::rspack_core::rspack_sources::Source>> {
        {
          let mut cached_generated_code = self.cached_generated_code.read().expect("Failed to acquire read lock on cached_generated_code");
          if let Some(cached_generated_code) = (*cached_generated_code).as_ref() {
            return Ok(cached_generated_code.clone());
          }
        }
        let mut cached_generated_code = self.cached_generated_code.write().expect("Failed to acquire write lock on cached_generated_code");
        let source = self.generate_with_custom(compilation)?;
        *cached_generated_code = Some(source.clone());
        Ok(source)
      }
    }

    impl #impl_generics ::rspack_core::CustomSourceRuntimeModule for #name #ty_generics #where_clause {
      fn set_custom_source(&mut self, source: ::rspack_core::rspack_sources::BoxSource) -> () {
        self.custom_source = Some(source);
      }
      fn get_custom_source(&self) -> Option<::rspack_core::rspack_sources::BoxSource> {
        self.custom_source.clone()
      }
      fn get_constructor_name(&self) -> String {
        String::from(stringify!(#name))
      }
    }

    impl #impl_generics rspack_collections::Identifiable for #name #ty_generics #where_clause {
      fn identifier(&self) -> rspack_collections::Identifier {
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
    impl #impl_generics ::rspack_core::Module for #name #ty_generics #where_clause {
      fn module_type(&self) -> &::rspack_core::ModuleType {
        &::rspack_core::ModuleType::Runtime
      }

      fn source_types(&self) -> &[::rspack_core::SourceType] {
        &[::rspack_core::SourceType::JavaScript]
      }

      fn size(&self, _source_type: Option<&::rspack_core::SourceType>, compilation: Option<&::rspack_core::Compilation>) -> f64 {
        match compilation {
          Some(compilation) => self.get_generated_code(compilation).ok().map(|source| source.size() as f64).unwrap_or(0f64),
          None => 0f64
        }
      }

      fn readable_identifier(&self, _context: &::rspack_core::Context) -> std::borrow::Cow<str> {
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

      fn code_generation(
        &self,
        compilation: &::rspack_core::Compilation,
        _runtime: Option<&::rspack_core::RuntimeSpec>,
        _: Option<::rspack_core::ConcatenationScope>,
      ) -> rspack_error::Result<::rspack_core::CodeGenerationResult> {
        let mut result = ::rspack_core::CodeGenerationResult::default();
        result.add(::rspack_core::SourceType::Runtime, self.get_generated_code(compilation)?);
        Ok(result)
      }

      fn update_hash(
        &self,
        hasher: &mut dyn std::hash::Hasher,
        compilation: &::rspack_core::Compilation,
        _runtime: Option<&::rspack_core::RuntimeSpec>,
      ) -> ::rspack_error::Result<()> {
        use rspack_util::ext::DynHash;
        self.name().dyn_hash(hasher);
        self.stage().dyn_hash(hasher);
        if self.full_hash() || self.dependent_hash() {
          self.generate_with_custom(compilation)?.dyn_hash(hasher);
        } else {
          self.get_generated_code(compilation)?.dyn_hash(hasher);
        }
        Ok(())
      }
    }

    impl #impl_generics rspack_error::Diagnosable for #name  #ty_generics #where_clause {
      fn add_diagnostic(&mut self, _diagnostic: rspack_error::Diagnostic) {
        unreachable!()
      }
      fn add_diagnostics(&mut self, _diagnostics: Vec<rspack_error::Diagnostic>) {
        unreachable!()
      }
      fn diagnostics(&self) -> std::borrow::Cow<[rspack_error::Diagnostic]> {
        std::borrow::Cow::Owned(vec![])
      }
    }

    impl #impl_generics ::rspack_util::source_map::ModuleSourceMapConfig for #name #ty_generics #where_clause {
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
