use cow_utils::CowUtils;
use heck::ToSnakeCase;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error, Fields, LitStr, Result, spanned::Spanned};

enum Fallback {
  Unit(syn::Ident),
  Newtype(syn::Ident),
}

#[derive(Clone, Copy, Default)]
enum RenameRule {
  Lower,
  Upper,
  Pascal,
  Camel,
  #[default]
  Snake,
  ScreamingSnake,
  Kebab,
  ScreamingKebab,
}

impl RenameRule {
  fn parse(value: &LitStr) -> Result<Self> {
    match value.value().as_str() {
      "lowercase" => Ok(Self::Lower),
      "UPPERCASE" => Ok(Self::Upper),
      "PascalCase" => Ok(Self::Pascal),
      "camelCase" => Ok(Self::Camel),
      "snake_case" => Ok(Self::Snake),
      "SCREAMING_SNAKE_CASE" => Ok(Self::ScreamingSnake),
      "kebab-case" => Ok(Self::Kebab),
      "SCREAMING-KEBAB-CASE" => Ok(Self::ScreamingKebab),
      _ => Err(Error::new(
        value.span(),
        "unknown rename rule, expected one of `lowercase`, `UPPERCASE`, `PascalCase`, \
         `camelCase`, `snake_case`, `SCREAMING_SNAKE_CASE`, `kebab-case`, or \
         `SCREAMING-KEBAB-CASE`",
      )),
    }
  }

  fn apply(self, value: &str) -> String {
    let snake_case = || value.to_snake_case();
    match self {
      Self::Lower => value.cow_to_ascii_lowercase().into_owned(),
      Self::Upper => value.cow_to_ascii_uppercase().into_owned(),
      Self::Pascal => value.to_string(),
      Self::Camel => {
        let mut chars = value.chars();
        chars
          .next()
          .map(|first| first.to_lowercase().collect::<String>() + chars.as_str())
          .unwrap_or_default()
      }
      Self::Snake => snake_case(),
      Self::ScreamingSnake => snake_case().cow_to_ascii_uppercase().into_owned(),
      Self::Kebab => snake_case().cow_replace('_', "-").into_owned(),
      Self::ScreamingKebab => snake_case()
        .cow_replace('_', "-")
        .cow_to_ascii_uppercase()
        .into_owned(),
    }
  }
}

pub fn expand(input: DeriveInput) -> Result<TokenStream> {
  let name = &input.ident;
  let Data::Enum(data) = &input.data else {
    return Err(Error::new(
      input.span(),
      "StringEnum can only be derived for enums",
    ));
  };

  let mut rename_all = None;
  for attribute in &input.attrs {
    if !attribute.path().is_ident("string_enum") {
      continue;
    }
    attribute.parse_nested_meta(|meta| {
      if meta.path.is_ident("rename_all") {
        if rename_all.is_some() {
          return Err(meta.error("duplicate string_enum rename_all option"));
        }
        let value = meta.value()?.parse::<LitStr>()?;
        rename_all = Some(RenameRule::parse(&value)?);
        Ok(())
      } else {
        Err(meta.error("unsupported string_enum container option"))
      }
    })?;
  }
  let rename_all = rename_all.unwrap_or_default();

  let mut fallback = None;
  let mut mappings = Vec::new();

  for variant in &data.variants {
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
      let fallback_variant = match &variant.fields {
        Fields::Unit => Fallback::Unit(variant.ident.clone()),
        Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
          Fallback::Newtype(variant.ident.clone())
        }
        _ => {
          return Err(Error::new(
            variant.fields.span(),
            "string enum fallback must be a unit variant or contain exactly one unnamed field",
          ));
        }
      };
      if fallback.replace(fallback_variant).is_some() {
        return Err(Error::new(
          variant.span(),
          "string enum must have exactly one fallback variant",
        ));
      }
      continue;
    }

    if !matches!(variant.fields, Fields::Unit) {
      return Err(Error::new(
        variant.fields.span(),
        "string enum non-fallback variants must not contain fields",
      ));
    }

    let value = rename.unwrap_or_else(|| {
      LitStr::new(
        &rename_all.apply(&variant.ident.to_string()),
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

  let (as_str_return_type, fallback_as_str_arm, fallback_from_str) = match fallback {
    Fallback::Unit(variant) => (
      quote!(&'static str),
      quote! {
        Self::#variant => unreachable!("string enum fallback has no string representation"),
      },
      quote!(Self::#variant),
    ),
    Fallback::Newtype(variant) => (
      quote!(&str),
      quote! {
        Self::#variant(value) => value,
      },
      quote!(Self::#variant(value.into())),
    ),
  };

  Ok(quote! {
    impl #impl_generics #name #type_generics #where_clause {
      pub fn as_str(&self) -> #as_str_return_type {
        match self {
          #(#as_str_arms)*
          #fallback_as_str_arm
        }
      }
    }

    impl #impl_generics ::std::convert::From<&str> for #name #type_generics #where_clause {
      fn from(value: &str) -> Self {
        match value {
          #(#from_str_arms)*
          _ => #fallback_from_str,
        }
      }
    }
  })
}
