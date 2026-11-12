use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, LitStr, Path, ext::IdentExt};

use crate::field_names::snake_to_camel;

pub(crate) fn expand(input: DeriveInput) -> syn::Result<TokenStream> {
  let Data::Struct(data) = &input.data else {
    return Err(syn::Error::new_spanned(input, "expected a struct"));
  };
  let Fields::Named(fields) = &data.fields else {
    return Err(syn::Error::new_spanned(
      &data.fields,
      "expected named fields",
    ));
  };
  let value = quote!(::rspack_napi::owned_value);
  let mut keys = Vec::with_capacity(fields.named.len());
  let mut initializers = Vec::with_capacity(fields.named.len());
  for field in &fields.named {
    let ident = field.ident.as_ref().expect("named field");
    let mut key = LitStr::new(&snake_to_camel(&ident.unraw().to_string()), ident.span());
    let mut convert: Option<Path> = None;
    for attr in field
      .attrs
      .iter()
      .filter(|attr| attr.path().is_ident("js_value"))
    {
      attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("rename") {
          key = meta.value()?.parse()?;
        } else if meta.path.is_ident("with") {
          convert = Some(meta.value()?.parse()?);
        } else {
          return Err(meta.error("expected rename or with"));
        }
        Ok(())
      })?;
    }
    let read = match convert {
      Some(convert) => quote!(#value::read_optional_field_with(fields, #key, #convert)?),
      None => quote!(#value::read_field(fields, #key)?),
    };
    keys.push(key);
    initializers.push(quote!(#ident: #read));
  }

  let name = &input.ident;
  let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
  Ok(quote! {
    impl #impl_generics #value::FromObject for #name #ty_generics #where_clause {
      const FIELDS: &'static [&'static str] = &[#(#keys),*];

      fn from_object(fields: #value::ObjectRef<'_>) -> ::rspack_error::Result<Self> {
        Ok(Self { #(#initializers),* })
      }
    }

    impl #impl_generics #value::FromValue for #name #ty_generics #where_clause {
      fn from_value(value: #value::ValueRef<'_>) -> ::std::option::Option<Self> {
        <Self as #value::FromObject>::from_object(value.object()?).ok()
      }
    }
  })
}
