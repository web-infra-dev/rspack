use proc_macro::TokenStream;
use quote::quote;
use syn::{Ident, ItemImpl, Type, parse_quote, spanned::Spanned};

use super::DynArgs;

/// For impl block providing trait or associated items: `impl<A> Trait
/// for Data<A> { ... }`.
pub fn impl_impl(mut input: ItemImpl, args: DynArgs) -> TokenStream {
  if input.generics.type_params().next().is_some() {
    // not support for generics
    input.items.push(parse_quote! {
        fn __dyn_id(&self) -> u64 {
            panic!("#[cacheable_dyn] not support for impl with generics");
        }
    });
    return quote! { #input }.into();
  }
  let crate_path = &args.crate_path;
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
            #dyn_id_ident
        }
  });

  quote! {
      const #dyn_id_ident: u64 = #crate_path::xxhash_rust::const_xxh64::xxh64(concat!(module_path!(), ":", line!()).as_bytes(), 0);

      #input

      const _: () = {
          use #crate_path::__private::{
              inventory,
              rkyv::{ptr_meta, ArchiveUnsized, Archived, Deserialize, DeserializeUnsized},
          };
          use #crate_path::{
              r#dyn::{
                  validation::{default_check_bytes_dyn, CheckBytesEntry},
                  DeserializeDyn, DynEntry, VTablePtr,
              },
              Error, Deserializer,
          };

          const fn get_vtable() -> VTablePtr {
              VTablePtr::new(ptr_meta::metadata(
                  core::ptr::null::<Archived<#target_ident>>() as *const <dyn #trait_ident as ArchiveUnsized>::Archived
              ))
          }
          inventory::submit! { DynEntry::new(#dyn_id_ident, get_vtable()) }
          inventory::submit! { CheckBytesEntry::new(get_vtable(), default_check_bytes_dyn::<Archived<#target_ident>>) }

          impl DeserializeDyn<dyn #trait_ident> for #archived_target_ident
          where
              #archived_target_ident: Deserialize<#target_ident, Deserializer>,
          {
              fn deserialize_dyn(
                  &self,
                  deserializer: &mut Deserializer,
                  out: *mut dyn #trait_ident
              ) -> Result<(), Error> {
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
