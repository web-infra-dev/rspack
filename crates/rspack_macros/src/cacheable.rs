use proc_macro::TokenStream;
use quote::quote;
use syn::{
  parse::{Parse, ParseStream},
  parse_macro_input, Item, Result, Token,
};

mod kw {
  syn::custom_keyword!(with);
}
pub struct CacheableArgs {
  pub with: Option<syn::Path>,
}
impl Parse for CacheableArgs {
  fn parse(input: ParseStream) -> Result<Self> {
    let mut with = None;

    let mut needs_punct = false;
    while !input.is_empty() {
      if needs_punct {
        input.parse::<Token![,]>()?;
      }

      if input.peek(kw::with) {
        if with.is_some() {
          return Err(input.error("duplicate with argument"));
        }

        input.parse::<kw::with>()?;
        input.parse::<Token![=]>()?;
        with = Some(input.parse::<syn::Path>()?);
      } else {
        return Err(
          input.error("expected serialize = \"...\" or deserialize = \"...\" parameters"),
        );
      }

      needs_punct = true;
    }

    Ok(Self { with })
  }
}

pub fn impl_cacheable(tokens: TokenStream) -> TokenStream {
  let input = parse_macro_input!(tokens as Item);
  quote! {
      #[derive(
          rspack_cacheable::__private::rkyv::Archive,
          rspack_cacheable::__private::rkyv::Deserialize,
          rspack_cacheable::__private::rkyv::Serialize
      )]
      #[archive(check_bytes, crate="rspack_cacheable::__private::rkyv")]
      #input
  }
  .into()
}

pub fn impl_cacheable_with(tokens: TokenStream, with: syn::Path) -> TokenStream {
  let input = parse_macro_input!(tokens as Item);
  // TODO use _impl_generics, _ty_generics, _where_clause
  let (ident, _impl_generics, _ty_generics, _where_clause) = match &input {
    Item::Enum(input) => {
      let (a, b, c) = input.generics.split_for_impl();
      (&input.ident, a, b, c)
    }
    Item::Struct(input) => {
      let (a, b, c) = input.generics.split_for_impl();
      (&input.ident, a, b, c)
    }
    _ => panic!("expect enum or struct"),
  };
  let archived = quote! {<#with as rkyv::with::ArchiveWith<#ident>>::Archived};
  let resolver = quote! {<#with as rkyv::with::ArchiveWith<#ident>>::Resolver};
  let rkyv_with = quote! {rkyv::with::With<#ident, #with>};
  quote! {
      #input
      #[allow(non_upper_case_globals)]
      const _: () = {
          use rspack_cacheable::__private::rkyv;
          impl rkyv::Archive for #ident {
              type Archived = #archived;
              type Resolver = #resolver;
              #[inline]
              unsafe fn resolve(&self, pos: usize, resolver: Self::Resolver, out: *mut Self::Archived) {
                  <#rkyv_with>::cast(self).resolve(pos, resolver, out)
              }
          }
          impl<S> rkyv::Serialize<S> for #ident
          where
              #rkyv_with: rkyv::Serialize<S>,
              S: rkyv::Fallible + ?Sized,
          {
              #[inline]
              fn serialize(&self, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
                  <#rkyv_with>::cast(self).serialize(serializer)
              }
          }
          impl<D: rkyv::Fallible + ?Sized> rkyv::Deserialize<#ident, D> for #archived
          where
              #rkyv_with: rkyv::Archive,
              rkyv::Archived<#rkyv_with>: rkyv::Deserialize<#rkyv_with, D>,
          {
              #[inline]
              fn deserialize(&self, _deserializer: &mut D) -> Result<#ident, D::Error> {
                  Ok(
                      rkyv::Deserialize::<#rkyv_with, D>::deserialize(
                          self,
                          _deserializer,
                      )?.into_inner()
                  )
              }
          }
      };
  }
  .into()
}
