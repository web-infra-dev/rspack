use heck::ToSnakeCase;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error, Fields, LitStr, Result, spanned::Spanned};

pub fn expand(input: DeriveInput) -> Result<TokenStream> {
  let name = &input.ident;
  let Data::Enum(data) = &input.data else {
    return Err(Error::new(
      input.span(),
      "StringEnum can only be derived for enums",
    ));
  };

  let mut fallback = None;
  let mut mappings = Vec::new();

  for variant in &data.variants {
    if !matches!(variant.fields, Fields::Unit) {
      return Err(Error::new(
        variant.fields.span(),
        "string enum variants must not contain fields",
      ));
    }

    let mut is_fallback = false;
    let mut rename = None;
    for attribute in &variant.attrs {
      if !attribute.path().is_ident("string_enum") {
        continue;
      }
      attribute.parse_nested_meta(|meta| {
        if meta.path.is_ident("fallback") {
          if is_fallback {
            return Err(meta.error("duplicate string_enum fallback option"));
          }
          is_fallback = true;
          Ok(())
        } else if meta.path.is_ident("rename") {
          if rename.is_some() {
            return Err(meta.error("duplicate string_enum rename option"));
          }
          rename = Some(meta.value()?.parse::<LitStr>()?);
          Ok(())
        } else {
          Err(meta.error("unsupported string_enum option"))
        }
      })?;
    }

    if is_fallback {
      if rename.is_some() {
        return Err(Error::new(
          variant.span(),
          "string enum fallback cannot be renamed",
        ));
      }
      if fallback.replace(variant.ident.clone()).is_some() {
        return Err(Error::new(
          variant.span(),
          "string enum must have exactly one fallback variant",
        ));
      }
      continue;
    }

    let value = rename.unwrap_or_else(|| {
      LitStr::new(
        &variant.ident.to_string().to_snake_case(),
        variant.ident.span(),
      )
    });
    let cfg_attrs = variant
      .attrs
      .iter()
      .filter(|attr| attr.path().is_ident("cfg"))
      .cloned()
      .collect::<Vec<_>>();
    mappings.push((variant.ident.clone(), value, cfg_attrs));
  }

  let fallback = fallback.ok_or_else(|| {
    Error::new(
      input.ident.span(),
      "string enum must have exactly one #[string_enum(fallback)] variant",
    )
  })?;
  let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

  let as_str_arms = mappings.iter().map(|(variant, value, cfg_attrs)| {
    quote! {
      #(#cfg_attrs)*
      Self::#variant => #value,
    }
  });
  let from_str_arms = mappings.iter().map(|(variant, value, cfg_attrs)| {
    quote! {
      #(#cfg_attrs)*
      #value => Self::#variant,
    }
  });

  Ok(quote! {
    impl #impl_generics #name #type_generics #where_clause {
      pub fn as_str(&self) -> &'static str {
        match self {
          #(#as_str_arms)*
          Self::#fallback => unreachable!("string enum fallback has no string representation"),
        }
      }
    }

    impl #impl_generics ::std::convert::From<&str> for #name #type_generics #where_clause {
      fn from(value: &str) -> Self {
        match value {
          #(#from_str_arms)*
          _ => Self::#fallback,
        }
      }
    }
  })
}
