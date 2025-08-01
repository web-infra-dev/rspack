use proc_macro::TokenStream;
use quote::quote;
use syn::{Field, Item, Token, Type, parse_macro_input, parse_quote, visit_mut::VisitMut};

use super::CacheableArgs;

/// A visitor to collect #[cacheable] info on field
#[derive(Default)]
struct FieldAttrVisitor {
  /// Whether any field set #[cacheable(omit_bounds)]
  omit_bounds: bool,
}

impl VisitMut for FieldAttrVisitor {
  fn visit_field_mut(&mut self, f: &mut Field) {
    let mut with_info = None;
    let mut omit_bounds = false;
    f.attrs.retain(|item| {
      if item.path().is_ident("cacheable") {
        let _ = item.parse_nested_meta(|meta| {
          if meta.path.is_ident("with") {
            meta.input.parse::<Token![=]>()?;
            with_info = Some(meta.input.parse::<Type>()?);
            return Ok(());
          }
          if meta.path.is_ident("omit_bounds") {
            omit_bounds = true;
            return Ok(());
          }
          Err(meta.error("unrecognized cacheable arguments"))
        });
        false
      } else {
        true
      }
    });

    // add rkyv with
    if let Some(with_info) = with_info {
      f.attrs.push(parse_quote!(#[rkyv(with=#with_info)]));
    }
    // add rkyv omit_bounds
    if omit_bounds {
      self.omit_bounds = true;
      f.attrs.push(parse_quote!(#[rkyv(omit_bounds)]));
    }
  }
}

/// impl #[cacheable] without with args
pub fn impl_cacheable(tokens: TokenStream, args: CacheableArgs) -> TokenStream {
  let mut input = parse_macro_input!(tokens as Item);

  let mut visitor = FieldAttrVisitor::default();
  visitor.visit_item_mut(&mut input);

  let crate_path = &args.crate_path;
  let archived_impl_hash = if args.hashable {
    quote! {#[rkyv(derive(Hash, PartialEq, Eq))]}
  } else {
    quote! {}
  };
  let bounds = if visitor.omit_bounds {
    quote! {
        #[rkyv(serialize_bounds(
            __S: #crate_path::__private::rkyv::ser::Writer + #crate_path::__private::rkyv::ser::Allocator + #crate_path::__private::rkyv::rancor::Fallible<Error = #crate_path::SerializeError>,
        ))]
        #[rkyv(deserialize_bounds(
            __D: #crate_path::__private::rkyv::rancor::Fallible<Error = #crate_path::DeserializeError>
        ))]
        #[rkyv(bytecheck(
            bounds(
                __C: #crate_path::__private::rkyv::validation::ArchiveContext + #crate_path::__private::rkyv::rancor::Fallible<Error = #crate_path::DeserializeError>,
            )
        ))]
    }
  } else {
    quote! {}
  };

  quote! {
      #[derive(
          #crate_path::__private::rkyv::Archive,
          #crate_path::__private::rkyv::Deserialize,
          #crate_path::__private::rkyv::Serialize
      )]
      #[rkyv(crate=#crate_path::__private::rkyv)]
      #archived_impl_hash
      #bounds
      #input
  }
  .into()
}
