use napi_derive_backend::BindgenResult;
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::Attribute;

pub fn expand(_attr: TokenStream, input: TokenStream) -> BindgenResult<TokenStream> {
  let mut item = syn::parse2::<syn::Item>(input)?;
  let mut tokens = TokenStream::new();

  if let syn::Item::Struct(ref mut struct_) = item {
    struct_
      .fields
      .iter_mut()
      .for_each(|field| find_and_remove_napi_attr(&mut field.attrs))
  }

  item.to_tokens(&mut tokens);

  Ok(tokens)
}

fn find_and_remove_napi_attr(attrs: &mut Vec<Attribute>) {
  loop {
    let napi_attr = attrs
      .iter()
      .enumerate()
      .find(|&(_, m)| m.path().segments[0].ident == "napi");

    let pos = match napi_attr {
      Some((pos, _raw_attr)) => pos,
      None => break,
    };

    attrs.remove(pos);
  }
}
