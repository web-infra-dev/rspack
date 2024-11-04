use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_quote, spanned::Spanned, GenericParam, Ident, ItemImpl, ItemTrait, Type};

/// For trait definition: `pub trait Iterator { ... }`
pub fn impl_trait(mut input: ItemTrait) -> TokenStream {
  let trait_ident = &input.ident;
  let trait_vis = &input.vis;
  let generic_params = input.generics.params.iter().map(|p| {
    // remove default value
    let mut p = p.clone();
    if let GenericParam::Type(param) = &mut p {
      param.eq_token = None;
      param.default = None;
    }
    quote! { #p }
  });
  let generic_params = quote! { #(#generic_params),* };
  let deserialize_trait_ident =
    Ident::new(&format!("Deserialize{trait_ident}"), trait_ident.span());
  let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

  input
    .supertraits
    .push(parse_quote! { rspack_cacheable::r#dyn::SerializeDyn });

  input.items.push(parse_quote! {
      #[doc(hidden)]
      fn __dyn_id(&self) -> u64;
  });

  quote! {
      #input

        const _: () = {
            use std::alloc::{Layout, LayoutError};

            use rspack_cacheable::__private::rkyv::{
                bytecheck::CheckBytes,
                ptr_meta,
                traits::{ArchivePointee, LayoutRaw},
                ArchiveUnsized, ArchivedMetadata, DeserializeUnsized, Portable, SerializeUnsized,
            };
            use rspack_cacheable::{
                r#dyn::{validation::CHECK_BYTES_REGISTRY, ArchivedDynMetadata, DeserializeDyn},
                DeserializeError, Deserializer, SerializeError, Serializer, Validator,
            };

            unsafe impl #impl_generics ptr_meta::Pointee for dyn #trait_ident #ty_generics #where_clause {
                type Metadata = ptr_meta::DynMetadata<Self>;
            }

            impl #ty_generics ArchiveUnsized for dyn #trait_ident #ty_generics #where_clause {
                type Archived = dyn #deserialize_trait_ident #ty_generics;

                fn archived_metadata(&self) -> ArchivedMetadata<Self> {
                    ArchivedDynMetadata::new(#trait_ident::__dyn_id(self))
                }
            }

            impl #ty_generics LayoutRaw for dyn #trait_ident #ty_generics #where_clause {
                fn layout_raw(
                    metadata: <Self as ptr_meta::Pointee>::Metadata,
                ) -> Result<Layout, LayoutError> {
                    Ok(metadata.layout())
                }
            }

            impl #ty_generics SerializeUnsized<Serializer<'_>> for dyn #trait_ident #ty_generics #where_clause {
                fn serialize_unsized(
                    &self,
                    serializer: &mut Serializer
                ) -> Result<usize, SerializeError> {
                    self.serialize_dyn(serializer)
                }
            }

            #trait_vis trait #deserialize_trait_ident #ty_generics: DeserializeDyn<dyn #trait_ident #ty_generics> + Portable #where_clause {}
            unsafe impl #ty_generics ptr_meta::Pointee for dyn #deserialize_trait_ident #ty_generics #where_clause {
                type Metadata = ptr_meta::DynMetadata<Self>;
            }

            impl<__T: DeserializeDyn<dyn #trait_ident #ty_generics> + Portable, #generic_params> #deserialize_trait_ident #ty_generics for __T #where_clause {}

            impl #ty_generics ArchivePointee for dyn #deserialize_trait_ident #ty_generics #where_clause {
                type ArchivedMetadata = ArchivedDynMetadata<Self>;

                fn pointer_metadata(
                    archived: &Self::ArchivedMetadata,
                ) -> <Self as ptr_meta::Pointee>::Metadata {
                    archived.lookup_metadata()
                }
            }


            impl #ty_generics DeserializeUnsized<dyn #trait_ident #ty_generics, Deserializer> for dyn #deserialize_trait_ident #ty_generics #where_clause {
                unsafe fn deserialize_unsized(
                    &self,
                    deserializer: &mut Deserializer,
                    out: *mut dyn #trait_ident #ty_generics
                ) -> Result<(), DeserializeError> {
                    self.deserialize_dyn(deserializer, out)
                }

                fn deserialize_metadata(&self) -> <dyn #trait_ident #ty_generics as ptr_meta::Pointee>::Metadata {
                    self.deserialized_pointer_metadata()
                }
            }

            impl #ty_generics LayoutRaw for dyn #deserialize_trait_ident #ty_generics #where_clause {
                fn layout_raw(
                    metadata: <Self as ptr_meta::Pointee>::Metadata,
                ) -> Result<Layout, LayoutError> {
                    Ok(metadata.layout())
                }
            }

            // CheckBytes
            unsafe impl #ty_generics CheckBytes<Validator<'_>> for dyn #deserialize_trait_ident #ty_generics #where_clause {
                #[inline]
                unsafe fn check_bytes(
                    value: *const Self,
                    context: &mut Validator,
                ) -> Result<(), DeserializeError> {
                    let vtable: usize = std::mem::transmute(ptr_meta::metadata(value));
                    if let Some(check_bytes_dyn) = CHECK_BYTES_REGISTRY.get(&vtable) {
                        check_bytes_dyn(value.cast(), context)?;
                        Ok(())
                    } else {
                        Err(DeserializeError::DynCheckBytesNotRegister)
                    }
                }
            }
        };
  }
  .into()
}

/// For impl block providing trait or associated items: `impl<A> Trait
/// for Data<A> { ... }`.
pub fn impl_impl(mut input: ItemImpl) -> TokenStream {
  let trait_ident = &input.trait_.as_ref().expect("should have trait ident").1;
  let trait_ident_str = trait_ident
    .segments
    .last()
    .expect("should have segments")
    .ident
    .to_string();
  let target_ident = &input.self_ty;
  let target_ident_str = match &*input.self_ty {
    Type::Path(inner) => inner
      .path
      .segments
      .last()
      .expect("should have segments")
      .ident
      .to_string(),
    _ => {
      panic!("cacheable_dyn unsupported this target")
    }
  };
  let archived_target_ident = Ident::new(&format!("Archived{target_ident_str}"), input.span());
  #[allow(clippy::disallowed_methods)]
  let dyn_id_ident = Ident::new(
    &format!(
      "__DYN_ID_{}_{}",
      target_ident_str.to_ascii_uppercase(),
      trait_ident_str.to_ascii_uppercase()
    ),
    input.span(),
  );

  input.items.push(parse_quote! {
        fn __dyn_id(&self) -> u64 {
            *#dyn_id_ident
        }
  });

  quote! {
      static #dyn_id_ident: std::sync::LazyLock<u64> = std::sync::LazyLock::new(|| {
          use std::hash::{DefaultHasher, Hash, Hasher};
          let mut hasher = DefaultHasher::new();
          module_path!().hash(&mut hasher);
          line!().hash(&mut hasher);
          hasher.finish()
      });

      #input

      const _: () = {
          use rspack_cacheable::__private::{
              inventory,
              rkyv::{ptr_meta, ArchiveUnsized, Archived, Deserialize, DeserializeUnsized},
          };
          use rspack_cacheable::{
              r#dyn::{
                  validation::{default_check_bytes_dyn, CheckBytesEntry},
                  DeserializeDyn, DynEntry,
              },
              DeserializeError, Deserializer,
          };

          fn get_vtable() -> usize {
              unsafe {
                  core::mem::transmute(ptr_meta::metadata(
                      core::ptr::null::<Archived<#target_ident>>() as *const <dyn #trait_ident as ArchiveUnsized>::Archived
                  ))
              }
          }
          inventory::submit! { DynEntry::new(*#dyn_id_ident, get_vtable()) }
          inventory::submit! { CheckBytesEntry::new(get_vtable(), default_check_bytes_dyn::<Archived<#target_ident>>) }

          impl DeserializeDyn<dyn #trait_ident> for #archived_target_ident
          where
              #archived_target_ident: Deserialize<#target_ident, Deserializer>,
          {
              fn deserialize_dyn(
                  &self,
                  deserializer: &mut Deserializer,
                  out: *mut dyn #trait_ident
              ) -> Result<(), DeserializeError> {
                  unsafe {
                      <Self as DeserializeUnsized<#target_ident, _>>::deserialize_unsized(self, deserializer, out.cast())
                  }
              }

              fn deserialized_pointer_metadata(&self) -> ptr_meta::DynMetadata<dyn #trait_ident> {
                  ptr_meta::metadata(core::ptr::null::<#target_ident>() as *const dyn #trait_ident)
              }
          }
      };
  }
  .into()
}

/// impl cacheable dyn when disable
pub fn disable_cacheable_dyn(input: TokenStream) -> TokenStream {
  input
}
