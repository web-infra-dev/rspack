use syn::{
  Result, Token,
  parse::{Parse, ParseStream},
  parse_quote,
};

/// #[cacheable] type-only args
pub struct DynArgs {
  pub crate_path: syn::Path,
}

impl Parse for DynArgs {
  fn parse(input: ParseStream) -> Result<Self> {
    let mut crate_path = parse_quote! { ::rspack_cacheable };

    let mut needs_punct = false;
    while !input.is_empty() {
      if needs_punct {
        input.parse::<Token![,]>()?;
      }

      if input.peek(syn::token::Crate) {
        input.parse::<syn::token::Crate>()?;
        input.parse::<Token![=]>()?;
        crate_path = input.parse::<syn::Path>()?;
      } else {
        return Err(input.error("unexpected #[cacheable] type-only parameters"));
      }

      needs_punct = true;
    }

    Ok(Self { crate_path })
  }
}
