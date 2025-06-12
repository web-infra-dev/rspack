use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
  parse::{Parse, ParseStream},
  parse_macro_input, Expr, Ident, LitStr, Token,
};

struct RegisterPluginInput {
  name: LitStr,
  _comma: Token![,],
  plugin: Expr,
}

impl Parse for RegisterPluginInput {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    Ok(RegisterPluginInput {
      name: input.parse()?,
      _comma: input.parse()?,
      plugin: input.parse()?,
    })
  }
}

#[proc_macro]
pub fn register_plugin(input: TokenStream) -> TokenStream {
  let RegisterPluginInput { name, plugin, .. } = parse_macro_input!(input as RegisterPluginInput);

  let plugin_register_ident: Ident =
    Ident::new(&format!("register_{}", name.value()), Span::call_site());

  let expanded = quote! {
      #[napi]
      fn #plugin_register_ident() {
          fn register<'a>(
              env: Env,
              options: Unknown<'a>,
          ) -> Result<rspack_core::BoxPlugin> {
            (#plugin)(env, options)
          }
          match rspack_binding_builder::register_custom_plugin(#name, register as rspack_binding_builder::CustomPluginBuilder) {
              Ok(_) => {}
              Err(err) => panic!("Cannot register plugins under the same name"),
          }
      }
  };

  TokenStream::from(expanded)
}
