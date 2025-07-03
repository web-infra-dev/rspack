use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{
  parse::{Parse, ParseStream},
  Expr, LitStr, Token,
};

pub struct RegisterPluginInput {
  name: LitStr,
  plugin: Expr,
}

impl Parse for RegisterPluginInput {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    let name = input.parse()?;
    <Token![,]>::parse(input)?;
    let plugin = input.parse()?;
    Ok(RegisterPluginInput { name, plugin })
  }
}

impl RegisterPluginInput {
  pub fn expand(self) -> TokenStream {
    let RegisterPluginInput { name, plugin } = self;

    let plugin_register_ident: Ident =
      Ident::new(&format!("register_{}", name.value()), Span::call_site());

    let expanded = quote! {
        #[napi]
        fn #plugin_register_ident() -> napi::Result<()> {
            fn register<'a>(
                env: Env,
                options: Unknown<'a>,
            ) -> Result<rspack_core::BoxPlugin> {
              (#plugin)(env, options)
            }
            let name = #name.to_string();
            rspack_binding_builder::register_custom_plugin(name, register as rspack_binding_builder::CustomPluginBuilder).map_err(|e| {
                napi::Error::from_reason(format!("Cannot register plugins under the same name: {}", #name))
            })
        }
    };

    expanded
  }
}
