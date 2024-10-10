use proc_macro::TokenStream;
use quote::quote;
use syn::{
  parse::{Parse, ParseStream},
  parse_macro_input, parse_quote,
  visit_mut::VisitMut,
  Field, Item, Result, Token, Type,
};

mod kw {
  syn::custom_keyword!(with);
  syn::custom_keyword!(hashable);
}
pub struct CacheableArgs {
  pub crate_path: syn::Path,
  pub with: Option<syn::Path>,
  pub hashable: bool,
}
impl Parse for CacheableArgs {
  fn parse(input: ParseStream) -> Result<Self> {
    let mut with = None;
    let mut crate_path = parse_quote! { ::rspack_cacheable };
    let mut hashable = false;

    let mut needs_punct = false;
    while !input.is_empty() {
      if needs_punct {
        input.parse::<Token![,]>()?;
      }

      if input.peek(syn::token::Crate) {
        input.parse::<syn::token::Crate>()?;
        input.parse::<Token![=]>()?;
        crate_path = input.parse::<syn::Path>()?;
      } else if input.peek(kw::with) {
        if with.is_some() {
          return Err(input.error("duplicate with argument"));
        }

        input.parse::<kw::with>()?;
        input.parse::<Token![=]>()?;
        with = Some(input.parse::<syn::Path>()?);
      } else if input.peek(kw::hashable) {
        input.parse::<kw::hashable>()?;
        hashable = true;
      } else {
        return Err(
          input.error("expected serialize = \"...\" or deserialize = \"...\" parameters"),
        );
      }

      needs_punct = true;
    }

    Ok(Self {
      crate_path,
      with,
      hashable,
    })
  }
}

#[derive(Default)]
struct FieldAttrVisitor {
  clean: bool,
}

impl VisitMut for FieldAttrVisitor {
  fn visit_field_mut(&mut self, f: &mut Field) {
    if self.clean {
      f.attrs.retain(|item| !item.path().is_ident("cacheable"));
      return;
    }

    for attr in f.attrs.iter_mut() {
      // replace #[cacheable(xxx)] to #[rkyv(xxx)] on fields
      let mut with_info = None;
      if attr.path().is_ident("cacheable") {
        attr
          .parse_nested_meta(|meta| {
            if meta.path.is_ident("with") {
              meta.input.parse::<Token![=]>()?;
              with_info = Some(meta.input.parse::<Type>()?);
              return Ok(());
            }
            Err(meta.error("unrecognized cacheable arguments"))
          })
          .expect("parse nested meta failed");
      }

      if let Some(with_info) = with_info {
        *attr = parse_quote!(#[rkyv(with=#with_info)]);
      }
    }
  }
}

pub fn impl_cacheable(tokens: TokenStream, args: CacheableArgs) -> TokenStream {
  let mut input = parse_macro_input!(tokens as Item);

  let mut visitor = FieldAttrVisitor::default();
  visitor.visit_item_mut(&mut input);

  let archived_impl_hash = if args.hashable {
    quote! {#[rkyv(derive(Hash, PartialEq, Eq))]}
  } else {
    quote! {}
  };
  let crate_path = &args.crate_path;

  quote! {
      #[derive(
          rspack_cacheable::__private::rkyv::Archive,
          rspack_cacheable::__private::rkyv::Deserialize,
          rspack_cacheable::__private::rkyv::Serialize
      )]
      #[rkyv(crate=#crate_path::__private::rkyv)]
      #archived_impl_hash
      #input
  }
  .into()
}

pub fn impl_cacheable_with(tokens: TokenStream, args: CacheableArgs) -> TokenStream {
  let mut input = parse_macro_input!(tokens as Item);

  let mut visitor = FieldAttrVisitor { clean: true };
  visitor.visit_item_mut(&mut input);

  let crate_path = args.crate_path;
  let with = args.with;

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
          impl Archive for #ident {
              type Archived = #archived;
              type Resolver = #resolver;
              #[inline]
              fn resolve(&self, resolver: Self::Resolver, out: Place<Self::Archived>) {
                  <#with as ArchiveWith<#ident>>::resolve_with(self, resolver, out)
              }
          }
          impl<S> Serialize<S> for #ident
          where
              S: Fallible + ?Sized,
              #with: SerializeWith<#ident, S>
          {
              #[inline]
              fn serialize(&self, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
                  #with::serialize_with(self, serializer)
              }
          }
          impl<D> Deserialize<#ident, D> for #archived
          where
              D: Fallible + ?Sized,
              #with: DeserializeWith<#archived, #ident, D>
          {
              #[inline]
              fn deserialize(&self, deserializer: &mut D) -> Result<#ident, D::Error> {
                  #with::deserialize_with(self, deserializer)
              }
          }
      };
  }
  .into()
}
