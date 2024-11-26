use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_quote, visit_mut::VisitMut, Field, Item, Token, Type};

use super::CacheableArgs;

/// A visitor to add #[cacheable(omit_bounds)] and collect #[cacheable(with=...)] info on field
#[derive(Default)]
struct FieldAttrVisitor {
  /// with info collected
  ///
  /// # Example
  ///
  /// ```rust,ignore
  /// #[cacheable]
  /// struct Test {
  ///   #[cacheable(with=AsMap<AsCacheable, AsPreset>)]
  ///   test_field: HashMap<String, Atom>,
  /// }
  ///
  /// // with_info is vec![(AsMap<AsCacheable, AsPreset>, HashMap<String, Atom>)]
  /// ```
  with_info: Vec<(Type, Type)>,
}

impl VisitMut for FieldAttrVisitor {
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
      self.with_info.push((with_info, f.ty.clone()));
    }
    // add rkyv omit_bounds
    f.attrs.push(parse_quote!(#[rkyv(omit_bounds)]));
  }
}

/// impl #[cacheable] with `as` args
pub fn impl_cacheable_as(tokens: TokenStream, args: CacheableArgs) -> TokenStream {
  let mut input = parse_macro_input!(tokens as Item);

  let mut visitor = FieldAttrVisitor::default();
  visitor.visit_item_mut(&mut input);

  let crate_path = &args.crate_path;
  let r#as = &args.r#as;

  let serialize_bounds = visitor
    .with_info
    .iter()
    .map(|(with, ty)| quote!(#with: #crate_path::__private::rkyv::with::SerializeWith<#ty, __S>));
  let serialize_bounds = quote! { #(#serialize_bounds),* };

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
          #serialize_bounds
      ))]
      #input
  }
  .into()
}
