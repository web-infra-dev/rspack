use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_quote, visit_mut::VisitMut, Field, Item, Token, Type};

use super::CacheableArgs;

/// A visitor to add #[cacheable(omit_bounds)] on field
struct AddFieldAttrVisitor;

impl VisitMut for AddFieldAttrVisitor {
  fn visit_field_mut(&mut self, f: &mut Field) {
    let mut with_info = None;
    f.attrs.retain(|item| {
      if item.path().is_ident("cacheable") {
        let _ = item.parse_nested_meta(|meta| {
          if meta.path.is_ident("with") {
            meta.input.parse::<Token![=]>()?;
            with_info = Some(meta.input.parse::<Type>()?);
            return Ok(());
          }
          if meta.path.is_ident("omit_bounds") {
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
    f.attrs.push(parse_quote!(#[rkyv(omit_bounds)]));
  }
}

/// impl #[cacheable] with `as` args
pub fn impl_cacheable_as(tokens: TokenStream, args: CacheableArgs) -> TokenStream {
  let mut input = parse_macro_input!(tokens as Item);

  let mut visitor = AddFieldAttrVisitor;
  visitor.visit_item_mut(&mut input);

  let crate_path = &args.crate_path;
  let r#as = &args.r#as;

  quote! {
      #[derive(
          #crate_path::__private::rkyv::Archive,
          #crate_path::__private::rkyv::Serialize
      )]
      #[rkyv(
          crate=#crate_path::__private::rkyv,
          as=#crate_path::__private::rkyv::Archived<#r#as>
      )]
      #[rkyv(serialize_bounds(
          __S: #crate_path::__private::rkyv::ser::Writer + #crate_path::__private::rkyv::ser::Allocator + #crate_path::__private::rkyv::rancor::Fallible<Error = #crate_path::SerializeError>,
      ))]
      #input
  }
  .into()
}
