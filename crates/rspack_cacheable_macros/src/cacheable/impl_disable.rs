use proc_macro::TokenStream;
use quote::quote;
use syn::{Field, Item, Token, Type, parse_macro_input, visit_mut::VisitMut};

use super::CacheableArgs;

/// A visitor to collect #[cacheable(with=...)] info on field
struct FieldAttrVisitor {
  /// with info collected
  ///
  /// # Example
  ///
  /// ```rust,ignore
  /// #[cacheable]
  /// struct Test {
  ///   #[cacheable(with=AsMap<AsCacheable, AsPreset>)]
  ///   test_field: HashMap<String, Atom>,
  /// }
  ///
  /// // with_info is vec![AsMap<AsCacheable, AsPreset>]
  /// ```
  with_info: Vec<Type>,
}

impl VisitMut for FieldAttrVisitor {
  fn visit_field_mut(&mut self, f: &mut Field) {
    f.attrs.retain(|item| {
      if item.path().is_ident("cacheable") {
        let _ = item.parse_nested_meta(|meta| {
          if meta.path.is_ident("with") {
            meta.input.parse::<Token![=]>()?;
            self.with_info.push(meta.input.parse::<Type>()?);
          }
          Ok(())
        });
        false
      } else {
        true
      }
    });
  }
}

/// impl cacheable when disable
pub fn impl_disable_cacheable(tokens: TokenStream, args: CacheableArgs) -> TokenStream {
  let mut input = parse_macro_input!(tokens as Item);
  let mut visitor = FieldAttrVisitor { with_info: vec![] };
  visitor.visit_item_mut(&mut input);
  if let Some(with_path) = args.with {
    visitor.with_info.push(with_path)
  }

  let ident = match &input {
    Item::Enum(input) => &input.ident,
    Item::Struct(input) => &input.ident,
    _ => panic!("expect enum or struct"),
  };
  let inner_fn_name = syn::Ident::new(
    #[allow(clippy::disallowed_methods)]
    &format!("_{}", ident.to_string().to_lowercase()),
    ident.span(),
  );
  let inner_type_list = visitor.with_info.iter();
  let inner_type_list = quote! { #(#inner_type_list),* };

  quote! {
      #input
      fn #inner_fn_name() {
          let _: Option<(#inner_type_list)> = None;
      }
  }
  .into()
}
