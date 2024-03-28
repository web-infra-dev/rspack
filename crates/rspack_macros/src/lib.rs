mod hook;
mod runtime_module;
mod source_map_config;

#[proc_macro_attribute]
pub fn impl_runtime_module(
  args: proc_macro::TokenStream,
  tokens: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
  runtime_module::impl_runtime_module(args, tokens)
}

#[proc_macro_attribute]
pub fn impl_source_map_config(
  args: proc_macro::TokenStream,
  tokens: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
  source_map_config::impl_source_map_config(args, tokens)
}

#[proc_macro_attribute]
pub fn plugin(
  _args: proc_macro::TokenStream,
  tokens: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
  let item = syn::parse_macro_input!(tokens as syn::ItemStruct);
  hook::expand_struct(item)
}

#[proc_macro_attribute]
pub fn plugin_hook(
  args: proc_macro::TokenStream,
  tokens: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
  let args = syn::parse_macro_input!(args as hook::HookArgs);
  let input: syn::ItemFn = syn::parse_macro_input!(tokens as syn::ItemFn);
  hook::expand_fn(args, input)
}
