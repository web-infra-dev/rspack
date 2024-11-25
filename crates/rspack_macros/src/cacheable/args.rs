use syn::{
  parse::{Parse, ParseStream},
  parse_quote, Result, Token,
};

mod kw {
  syn::custom_keyword!(with);
  syn::custom_keyword!(hashable);
}

/// #[cacheable] type-only args
pub struct CacheableArgs {
  pub crate_path: syn::Path,
  pub r#as: Option<syn::Path>,
  pub with: Option<syn::Path>,
  pub hashable: bool,
}

impl Parse for CacheableArgs {
  fn parse(input: ParseStream) -> Result<Self> {
    let mut crate_path = parse_quote! { ::rspack_cacheable };
    let mut r#as = None;
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
      } else if input.peek(syn::token::As) {
        input.parse::<syn::token::As>()?;
        input.parse::<Token![=]>()?;
        r#as = Some(input.parse::<syn::Path>()?);
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
      r#as,
      with,
      hashable,
    })
  }
}
