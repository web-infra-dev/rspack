use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, visit_mut::VisitMut, Field, Item};

/// A visitor to remove #[cacheable] on field
struct CleanFieldAttrVisitor;

impl VisitMut for CleanFieldAttrVisitor {
  fn visit_field_mut(&mut self, f: &mut Field) {
    f.attrs.retain(|item| !item.path().is_ident("cacheable"));
  }
}

/// impl cacheable when disable
pub fn disable_cacheable(tokens: TokenStream) -> TokenStream {
  let mut input = parse_macro_input!(tokens as Item);

  let mut visitor = CleanFieldAttrVisitor;
  visitor.visit_item_mut(&mut input);

  quote! {
      #input
  }
  .into()
}
