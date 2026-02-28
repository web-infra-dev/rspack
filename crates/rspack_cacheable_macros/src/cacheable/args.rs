use syn::{
  Result, Token,
  parse::{Parse, ParseStream},
  parse_quote,
};

mod kw {
  syn::custom_keyword!(with);
  syn::custom_keyword!(hashable);
}

/// #[cacheable] type-only args
pub struct CacheableArgs {
  pub crate_path: syn::Path,
  pub with: Option<syn::Type>,
  pub hashable: bool,
}

impl Parse for CacheableArgs {
  fn parse(input: ParseStream) -> Result<Self> {
    let mut crate_path = parse_quote! { ::rspack_cacheable };
    let mut with = None;
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
        with = Some(input.parse::<syn::Type>()?);
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
