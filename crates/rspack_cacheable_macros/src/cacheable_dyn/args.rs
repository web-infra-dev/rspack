use syn::{
  Result, Token,
  parse::{Parse, ParseStream},
  parse_quote,
};

mod kw {
  syn::custom_keyword!(unique_arc);
}

/// Options for `#[cacheable_dyn]` traits and implementations.
pub struct DynArgs {
  pub crate_path: syn::Path,
  pub unique_arc: bool,
}

impl Parse for DynArgs {
  fn parse(input: ParseStream) -> Result<Self> {
    let mut crate_path = parse_quote! { ::rspack_cacheable };
    let mut unique_arc = false;

    let mut needs_punct = false;
    while !input.is_empty() {
      if needs_punct {
        input.parse::<Token![,]>()?;
      }

      if input.peek(syn::token::Crate) {
        input.parse::<syn::token::Crate>()?;
        input.parse::<Token![=]>()?;
        crate_path = input.parse::<syn::Path>()?;
      } else if input.peek(kw::unique_arc) {
        input.parse::<kw::unique_arc>()?;
        unique_arc = true;
      } else {
        return Err(input.error("unexpected #[cacheable_dyn] parameters"));
      }

      needs_punct = true;
    }

    Ok(Self {
      crate_path,
      unique_arc,
    })
  }
}
