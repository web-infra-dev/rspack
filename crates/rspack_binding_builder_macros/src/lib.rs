mod register_plugin;

use proc_macro::TokenStream;
use syn::parse_macro_input;

#[proc_macro]
pub fn register_plugin(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as register_plugin::RegisterPluginInput);
  input.expand().into()
}
