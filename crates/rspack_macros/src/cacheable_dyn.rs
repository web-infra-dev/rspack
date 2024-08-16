use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_quote, spanned::Spanned, GenericParam, Ident, ItemImpl, ItemTrait, Type};

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
            use core::alloc::Layout;
            use std::alloc::LayoutError;

            use rspack_cacheable::__private::{
                ptr_meta,
                rkyv::{
                    validation::{validators::DefaultValidator, LayoutRaw},
                    ArchivePointee, ArchiveUnsized, ArchivedMetadata, CheckBytes, DeserializeUnsized,
                    SerializeUnsized,
                },
            };
            use rspack_cacheable::{
                r#dyn::{validation::CHECK_BYTES_REGISTRY, ArchivedDynMetadata, DeserializeDyn},
                CacheableDeserializer, CacheableSerializer, DeserializeError, SerializeError,
            };

            impl #impl_generics ptr_meta::Pointee for dyn #trait_ident #ty_generics #where_clause {
                type Metadata = ptr_meta::DynMetadata<Self>;
            }

            #trait_vis trait #deserialize_trait_ident #ty_generics: DeserializeDyn<dyn #trait_ident #ty_generics> {}
            impl #ty_generics ptr_meta::Pointee for dyn #deserialize_trait_ident #ty_generics {
                type Metadata = ptr_meta::DynMetadata<Self>;
            }

            impl<__O: DeserializeDyn<dyn #trait_ident #ty_generics>, #generic_params> #deserialize_trait_ident #ty_generics for __O {}

            impl #ty_generics ArchiveUnsized for dyn #trait_ident #ty_generics {
                type Archived = dyn #deserialize_trait_ident #ty_generics;
                type MetadataResolver = ();

                unsafe fn resolve_metadata(
                    &self,
                    _: usize,
                    _: Self::MetadataResolver,
                    out: *mut ArchivedMetadata<Self>,
                ) {
                    ArchivedDynMetadata::emplace(#trait_ident::__dyn_id(self), out);
                }
            }

            impl #ty_generics ArchivePointee for dyn #deserialize_trait_ident #ty_generics {
                type ArchivedMetadata = ArchivedDynMetadata<Self>;

                fn pointer_metadata(
                    archived: &Self::ArchivedMetadata,
                ) -> <Self as ptr_meta::Pointee>::Metadata {
                    archived.pointer_metadata()
                }
            }

            impl #ty_generics SerializeUnsized<CacheableSerializer> for dyn #trait_ident #ty_generics {
                fn serialize_unsized(
                    &self,
                    mut serializer: &mut CacheableSerializer
                ) -> Result<usize, SerializeError> {
                    self.serialize_dyn(&mut serializer)
                }

                fn serialize_metadata(
                    &self,
                    _: &mut CacheableSerializer
                ) -> Result<Self::MetadataResolver, SerializeError> {
                    Ok(())
                }
            }

            impl #ty_generics DeserializeUnsized<dyn #trait_ident #ty_generics, CacheableDeserializer> for dyn #deserialize_trait_ident #ty_generics {
                unsafe fn deserialize_unsized(
                    &self,
                    mut deserializer: &mut CacheableDeserializer,
                    mut alloc: impl FnMut(Layout) -> *mut u8,
                ) -> Result<*mut (), DeserializeError> {
                    self.deserialize_dyn(&mut deserializer, &mut alloc)
                }

                fn deserialize_metadata(
                    &self,
                    mut deserializer: &mut CacheableDeserializer,
                ) -> Result<<dyn #trait_ident #ty_generics as ptr_meta::Pointee>::Metadata, DeserializeError> {
                    self.deserialize_dyn_metadata(&mut deserializer)
                }
            }

            // CheckBytes
            impl #ty_generics LayoutRaw for dyn #deserialize_trait_ident #ty_generics {
                fn layout_raw(
                    metadata: <Self as ptr_meta::Pointee>::Metadata,
                ) -> Result<Layout, LayoutError> {
                    Ok(metadata.layout())
                }
            }
            impl #ty_generics CheckBytes<DefaultValidator<'_>> for dyn #deserialize_trait_ident #ty_generics {
                type Error = DeserializeError;
                #[inline]
                unsafe fn check_bytes<'a>(
                    value: *const Self,
                    context: &mut DefaultValidator<'_>,
                ) -> Result<&'a Self, Self::Error> {
                    let vtable: usize = core::mem::transmute(ptr_meta::metadata(value));
                    if let Some(check_bytes_dyn) = CHECK_BYTES_REGISTRY.get(&vtable) {
                        check_bytes_dyn(value.cast(), context)?;
                        Ok(&*value)
                    } else {
                        Err(DeserializeError::CheckBytesError)
                    }
                }
            }
        };
  }
  .into()
}

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
      panic!("cacheable_dyn unsupport this target")
    }
  };
  let archived_target_ident = Ident::new(&format!("Archived{}", target_ident_str), input.span());
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
      const #dyn_id_ident: std::sync::LazyLock<u64> = std::sync::LazyLock::new(|| {
          use std::hash::{DefaultHasher, Hash, Hasher};
          let mut hasher = DefaultHasher::new();
          module_path!().hash(&mut hasher);
          line!().hash(&mut hasher);
          hasher.finish()
      });

      #input

      const _: () = {
          use core::alloc::Layout;

          use rspack_cacheable::__private::{
              inventory, ptr_meta,
              rkyv::{ArchiveUnsized, Archived, Deserialize},
          };
          use rspack_cacheable::{
              r#dyn::{
                  validation::{default_check_bytes_dyn, CheckBytesEntry},
                  DeserializeDyn, DynEntry,
              },
              CacheableDeserializer, DeserializeError,
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
              #archived_target_ident: Deserialize<#target_ident, CacheableDeserializer>,
          {
              unsafe fn deserialize_dyn(
                  &self,
                  deserializer: &mut CacheableDeserializer,
                  alloc: &mut dyn FnMut(Layout) -> *mut u8,
              ) -> Result<*mut (), DeserializeError> {
                  let result = alloc(Layout::new::<#target_ident>()).cast::<#target_ident>();
                  assert!(!result.is_null());
                  result.write(self.deserialize(deserializer)?);
                  Ok(result as *mut ())
              }

              fn deserialize_dyn_metadata(
                  &self,
                  _: &mut CacheableDeserializer,
              ) -> Result<<dyn #trait_ident as ptr_meta::Pointee>::Metadata, DeserializeError> {
                  unsafe {
                      Ok(core::mem::transmute(ptr_meta::metadata(
                          core::ptr::null::<#target_ident>() as *const dyn #trait_ident,
                      )))
                  }
              }
          }
      };
  }
  .into()
}
