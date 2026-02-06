use proc_macro::TokenStream;
use quote::quote;
use syn::{Field, GenericParam, Item, parse_macro_input, visit_mut::VisitMut};

use super::CacheableArgs;

/// A visitor to remove #[cacheable] on field
struct CleanFieldAttrVisitor;

impl VisitMut for CleanFieldAttrVisitor {
  fn visit_field_mut(&mut self, f: &mut Field) {
    f.attrs.retain(|item| !item.path().is_ident("cacheable"));
  }
}

/// impl #[cacheable] with `with` args
pub(super) fn impl_cacheable_with(tokens: TokenStream, args: CacheableArgs) -> TokenStream {
  let mut input = parse_macro_input!(tokens as Item);

  let mut visitor = CleanFieldAttrVisitor;
  visitor.visit_item_mut(&mut input);

  let crate_path = args.crate_path;
  let with = args.with;

  let (ident, generics) = match &input {
    Item::Enum(input) => (&input.ident, &input.generics),
    Item::Struct(input) => (&input.ident, &input.generics),
    _ => panic!("expect enum or struct"),
  };
  let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
  let generic_params = generics.params.iter().map(|p| {
    // remove default value
    let mut p = p.clone();
    if let GenericParam::Type(param) = &mut p {
      param.eq_token = None;
      param.default = None;
    }
    quote! { #p }
  });
  let generic_params = quote! { #(#generic_params),* };

  let where_params = if let Some(where_clause) = &generics.where_clause {
    let params = where_clause.predicates.iter().map(|w| {
      quote! { #w }
    });
    quote! { #(#params),* }
  } else {
    quote! {}
  };

  let archived = quote! {<#with as rkyv::with::ArchiveWith<#ident #ty_generics>>::Archived};
  let resolver = quote! {<#with as rkyv::with::ArchiveWith<#ident #ty_generics>>::Resolver};
  quote! {
      #input
      #[allow(non_upper_case_globals)]
      const _: () = {
          use #crate_path::__private::rkyv;
          use rkyv::{
              rancor::Fallible,
              with::{ArchiveWith, DeserializeWith, SerializeWith},
              Archive, Deserialize, Place, Serialize
          };
          impl #impl_generics Archive for #ident #ty_generics #where_clause {
              type Archived = #archived;
              type Resolver = #resolver;
              #[inline]
              fn resolve(&self, resolver: Self::Resolver, out: Place<Self::Archived>) {
                  <#with as ArchiveWith<#ident #ty_generics>>::resolve_with(self, resolver, out)
              }
          }
          impl<__S, #generic_params> Serialize<__S> for #ident #ty_generics
          where
              __S: Fallible + ?Sized,
              #with: SerializeWith<#ident #ty_generics, __S>,
              #where_params
          {
              #[inline]
              fn serialize(&self, serializer: &mut __S) -> Result<Self::Resolver, __S::Error> {
                  #with::serialize_with(self, serializer)
              }
          }
          impl<__D, #generic_params> Deserialize<#ident #ty_generics, __D> for #archived
          where
              __D: Fallible + ?Sized,
              #with: DeserializeWith<#archived, #ident #ty_generics, __D>,
              #where_params
          {
              #[inline]
              fn deserialize(&self, deserializer: &mut __D) -> Result<#ident #ty_generics, __D::Error> {
                  #with::deserialize_with(self, deserializer)
              }
          }
      };
  }
  .into()
}
