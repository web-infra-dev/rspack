use std::sync::atomic::{AtomicBool, Ordering};

use napi_derive_backend::{BindgenResult, TryToTokens};
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{Attribute, Item};

use super::typedef;
use crate::parser::{ParseNapi, attrs::BindgenAttrs};

/// a flag indicate whether or never at least one `napi` macro has been expanded.
/// ```ignore
/// if BUILT_FLAG
///  .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
///  .is_ok() {
///   // logic on first macro expansion
/// }
/// ```
static BUILT_FLAG: AtomicBool = AtomicBool::new(false);

pub fn expand(attr: TokenStream, input: TokenStream) -> BindgenResult<TokenStream> {
  // logic on first macro expansion
  if let Ok(built) = BUILT_FLAG.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
  {
    if !built {
      typedef::prepare_type_def_file();
    }
  }

  let mut item = syn::parse2::<Item>(input)?;
  let opts: BindgenAttrs = syn::parse2(attr)?;
  let mut tokens = proc_macro2::TokenStream::new();
  if let Item::Mod(mut js_mod) = item {
    let js_name = opts.js_name().map_or_else(
      || js_mod.ident.to_string(),
      |(js_name, _)| js_name.to_owned(),
    );
    if let Some((_, mut items)) = js_mod.content.clone() {
      for item in items.iter_mut() {
        let mut empty_attrs = vec![];
        if let Some(item_opts) = replace_napi_attr_in_mod(
          js_name.clone(),
          match item {
            Item::Fn(ref mut function) => &mut function.attrs,
            Item::Struct(ref mut struct_) => &mut struct_.attrs,
            Item::Enum(ref mut enum_) => &mut enum_.attrs,
            Item::Const(ref mut const_) => &mut const_.attrs,
            Item::Impl(ref mut impl_) => &mut impl_.attrs,
            Item::Mod(mod_) => {
              let mod_in_mod = mod_
                .attrs
                .iter()
                .enumerate()
                .find(|(_, m)| m.path().is_ident("napi"));
              if mod_in_mod.is_some() {
                bail_span!(
                  mod_,
                  "napi module cannot be nested under another napi module"
                );
              } else {
                &mut empty_attrs
              }
            }
            _ => &mut empty_attrs,
          },
        ) {
          let napi = item.parse_napi(&mut tokens, &item_opts)?;
          item_opts.check_used()?;
          napi.try_to_tokens(&mut tokens)?;
          typedef::output_type_def(&napi);
        } else {
          item.to_tokens(&mut tokens);
        };
      }
      js_mod.content = None;
    };

    let js_mod_attrs: Vec<Attribute> = js_mod
      .attrs
      .clone()
      .into_iter()
      .filter(|attr| attr.path().is_ident("napi"))
      .collect();
    let mod_name = js_mod.ident;
    let visible = js_mod.vis;
    let mod_tokens = quote! { #(#js_mod_attrs)* #visible mod #mod_name { #tokens } };
    Ok(mod_tokens)
  } else {
    let napi = item.parse_napi(&mut tokens, &opts)?;
    opts.check_used()?;
    napi.try_to_tokens(&mut tokens)?;
    typedef::output_type_def(&napi);
    Ok(tokens)
  }
}

fn replace_napi_attr_in_mod(
  js_namespace: String,
  attrs: &mut Vec<syn::Attribute>,
) -> Option<BindgenAttrs> {
  let napi_attr = attrs
    .iter()
    .enumerate()
    .find(|(_, m)| m.path().is_ident("napi"));

  if let Some((index, napi_attr)) = napi_attr {
    // adds `namespace = #js_namespace` into `#[napi]` attribute
    let new_attr = match &napi_attr.meta {
      syn::Meta::Path(_) => {
        syn::parse_quote!(#[napi(namespace = #js_namespace)])
      }
      syn::Meta::List(list) => {
        let existing = list.tokens.clone();
        syn::parse_quote!(#[napi(#existing, namespace = #js_namespace)])
      }
      syn::Meta::NameValue(name_value) => {
        let existing = &name_value.value;
        syn::parse_quote!(#[napi(#existing, namespace = #js_namespace)])
      }
    };

    let struct_opts = BindgenAttrs::try_from(&new_attr).unwrap();
    attrs.remove(index);
    Some(struct_opts)
  } else {
    None
  }
}
