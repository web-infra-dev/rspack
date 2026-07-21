use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error, ExprPath, Fields, LitStr, Result};

/// Derives `from_ast_object` for plain data structs, extracting each field
/// from an AST object literal. See `utils/ast_object.rs` in
/// `rspack_plugin_javascript` for the runtime side.
pub fn expand_ast_object_derive(input: DeriveInput) -> Result<TokenStream> {
  let name = &input.ident;

  if !input.generics.params.is_empty() {
    return Err(Error::new_spanned(
      &input.generics,
      "AstObject does not support generic structs",
    ));
  }

  let mut rename_all_camel_case = false;
  for attr in &input.attrs {
    if !attr.path().is_ident("ast_object") {
      continue;
    }
    attr.parse_nested_meta(|meta| {
      if meta.path.is_ident("rename_all") {
        let value: LitStr = meta.value()?.parse()?;
        if value.value() == "camelCase" {
          rename_all_camel_case = true;
          Ok(())
        } else {
          Err(meta.error("only `camelCase` is supported"))
        }
      } else {
        Err(meta.error("unsupported attribute, expected `rename_all`"))
      }
    })?;
  }

  let fields = match &input.data {
    Data::Struct(data) => match &data.fields {
      Fields::Named(fields) => &fields.named,
      fields => {
        return Err(Error::new_spanned(
          fields,
          "AstObject requires a struct with named fields",
        ));
      }
    },
    data => {
      return Err(Error::new_spanned(
        &input.ident,
        match data {
          Data::Enum(_) => "AstObject does not support enums",
          Data::Union(_) => "AstObject does not support unions",
          Data::Struct(_) => unreachable!(),
        },
      ));
    }
  };

  let mut initializers = Vec::with_capacity(fields.len());
  for field in fields {
    let field_ident = field.ident.as_ref().expect("named field");
    let mut skip = false;
    let mut rename = None;
    let mut default_fn: Option<ExprPath> = None;
    for attr in &field.attrs {
      if !attr.path().is_ident("ast_object") {
        continue;
      }
      attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("skip") {
          skip = true;
          Ok(())
        } else if meta.path.is_ident("rename") {
          let value: LitStr = meta.value()?.parse()?;
          rename = Some(value.value());
          Ok(())
        } else if meta.path.is_ident("default") {
          let value: LitStr = meta.value()?.parse()?;
          default_fn = Some(value.parse()?);
          Ok(())
        } else {
          Err(meta.error("unsupported attribute, expected `skip`, `rename` or `default`"))
        }
      })?;
    }

    if skip {
      let default = default_value(default_fn);
      initializers.push(quote!(#field_ident: #default,));
      continue;
    }

    let key = rename.unwrap_or_else(|| {
      let ident = field_ident.to_string();
      if rename_all_camel_case {
        snake_to_camel_case(&ident)
      } else {
        ident
      }
    });
    let ty = &field.ty;
    let extract = quote! {
      crate::utils::ast_object::get_value_from_object::<#ty>(obj, &[#key])
    };
    let initializer = match default_fn {
      Some(default_fn) => quote!(#extract.unwrap_or_else(#default_fn)),
      None => quote!(#extract.unwrap_or_default()),
    };
    initializers.push(quote!(#field_ident: #initializer,));
  }

  Ok(quote! {
    impl #name {
      /// Extract the options from an AST object literal. Properties that are
      /// absent or not statically resolvable fall back to the field default.
      pub fn from_ast_object(obj: &::swc_experimental_ecma_ast::ObjectLit<'_>) -> Self {
        Self {
          #(#initializers)*
        }
      }
    }

    impl<'__ast> crate::utils::ast_object::FromAstExpr<'__ast> for #name {
      fn from_ast_expr(expr: &::swc_experimental_ecma_ast::Expr<'__ast>) -> Option<Self> {
        expr.as_object().map(Self::from_ast_object)
      }
    }
  })
}

fn default_value(default_fn: Option<ExprPath>) -> TokenStream {
  match default_fn {
    Some(default_fn) => quote!(#default_fn()),
    None => quote!(::core::default::Default::default()),
  }
}

fn snake_to_camel_case(ident: &str) -> String {
  let mut camel = String::with_capacity(ident.len());
  let mut uppercase_next = false;
  for ch in ident.chars() {
    if ch == '_' {
      uppercase_next = true;
    } else if uppercase_next {
      camel.extend(ch.to_uppercase());
      uppercase_next = false;
    } else {
      camel.push(ch);
    }
  }
  camel
}
