use proc_macro::TokenStream;
use quote::quote;
use syn::{
  parse::{Parse, ParseStream},
  parse_macro_input, parse_quote,
  visit_mut::VisitMut,
  Field, GenericParam, Item, Result, Token, Type,
};

mod kw {
  syn::custom_keyword!(with);
  syn::custom_keyword!(hashable);
}

/// #[cacheable] type-only args
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
        return Err(input.error("unexpected #[cacheable] type-only parameters"));
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

/// A visitor to transform #[cacheable] on field
#[derive(Default)]
struct FieldAttrVisitor {
  /// Remove all #[cacheable] attr on field
  clean: bool,
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

    // enable clean, just remove all cacheable attributes
    if self.clean {
      return;
    }
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

/// impl #[cacheable] with `with` args
pub fn impl_cacheable_with(tokens: TokenStream, args: CacheableArgs) -> TokenStream {
  let mut input = parse_macro_input!(tokens as Item);

  let mut visitor = FieldAttrVisitor {
    clean: true,
    ..Default::default()
  };
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

/// impl cacheable when disable
pub fn disable_cacheable(tokens: TokenStream) -> TokenStream {
  let mut input = parse_macro_input!(tokens as Item);

  let mut visitor = FieldAttrVisitor {
    clean: true,
    ..Default::default()
  };
  visitor.visit_item_mut(&mut input);

  quote! {
      #input
  }
  .into()
}
