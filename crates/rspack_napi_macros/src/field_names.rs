use quote::quote;
use syn::{parse_macro_input, Fields, ItemStruct};

pub fn field_names(
  _args: proc_macro::TokenStream,
  tokens: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
  let input = parse_macro_input!(tokens as ItemStruct);

  let struct_ident = &input.ident;

  let field_idents = match &input.fields {
    Fields::Named(named_fields) => named_fields
      .named
      .iter()
      .map(|f| {
        #[allow(clippy::unwrap_used)]
        let field_name = f.ident.as_ref().unwrap().to_string();
        let camel_field_name = snake_to_camel(&field_name);
        quote! {
            #camel_field_name.to_string()
        }
      })
      .collect::<Vec<_>>(),
    _ => Vec::new(),
  };

  let expanded = quote! {
      #input

      impl #struct_ident {
          pub fn field_names() -> ::std::vec::Vec<String> {
              ::std::vec![ #( #field_idents ),* ]
          }
      }
  };

  expanded.into()
}

fn snake_to_camel(s: &str) -> String {
  let mut out = String::new();
  let mut uppercase_next = false;
  for c in s.chars() {
    if c == '_' {
      uppercase_next = true;
    } else if uppercase_next {
      out.push(c.to_ascii_uppercase());
      uppercase_next = false;
    } else {
      out.push(c);
    }
  }
  out
}
