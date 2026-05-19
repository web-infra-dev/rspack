use proc_macro::TokenStream;
use quote::quote;
use syn::{GenericParam, Ident, ItemTrait, parse_quote};

use super::DynArgs;

/// For trait definition: `pub trait Iterator { ... }`
pub fn impl_trait(mut input: ItemTrait, args: DynArgs) -> TokenStream {
  let crate_path = &args.crate_path;
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
    .push(parse_quote! { #crate_path::r#dyn::SerializeDyn });

  input.items.push(parse_quote! {
      #[doc(hidden)]
      fn __dyn_id(&self) -> u64;
  });

  quote! {
      #input

        const _: () = {
            use std::alloc::{Layout, LayoutError};

            use #crate_path::__private::rkyv::{
                bytecheck::CheckBytes,
                ptr_meta,
                traits::{ArchivePointee, LayoutRaw},
                ArchiveUnsized, ArchivedMetadata, DeserializeUnsized, Portable, SerializeUnsized,
            };
            use #crate_path::{
                r#dyn::{validation::CHECK_BYTES_REGISTRY, ArchivedDynMetadata, DeserializeDyn, VTablePtr},
                Error, Deserializer, Serializer, Validator,
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
                ) -> Result<usize, Error> {
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
                ) -> Result<(), Error> {
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
                ) -> Result<(), Error> {
                    let vtable = VTablePtr::new(ptr_meta::metadata(value));
                    if let Some(check_bytes_dyn) = CHECK_BYTES_REGISTRY.get(&vtable) {
                        check_bytes_dyn(value.cast(), context)?;
                        Ok(())
                    } else {
                        Err(Error::DynCheckBytesNotRegister)
                    }
                }
            }
        };
  }
  .into()
}
