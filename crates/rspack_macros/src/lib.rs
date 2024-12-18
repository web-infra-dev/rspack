#![feature(try_find)]

mod cacheable;
mod cacheable_dyn;
mod hook;
mod merge;
mod plugin;
mod rspack_version;
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
  plugin::expand_struct(item)
}

#[proc_macro_attribute]
pub fn plugin_hook(
  args: proc_macro::TokenStream,
  tokens: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
  let args = syn::parse_macro_input!(args as plugin::HookArgs);
  let input = syn::parse_macro_input!(tokens as syn::ItemFn);
  plugin::expand_fn(args, input)
}

#[proc_macro]
pub fn define_hook(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let input = syn::parse_macro_input!(input as hook::DefineHookInput);
  match input.expand() {
    syn::Result::Ok(tt) => tt,
    syn::Result::Err(err) => err.to_compile_error(),
  }
  .into()
}

#[proc_macro_derive(MergeFrom, attributes(merge_from))]
pub fn merge_from_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let input = syn::parse_macro_input!(input as syn::DeriveInput);
  let output = merge::expand_merge_from_derive(input);
  match output {
    syn::Result::Ok(tt) => tt,
    syn::Result::Err(err) => err.to_compile_error(),
  }
  .into()
}

#[proc_macro_attribute]
pub fn cacheable(
  args: proc_macro::TokenStream,
  tokens: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
  cacheable::cacheable(args, tokens)
}

#[proc_macro_attribute]
pub fn disable_cacheable(
  _args: proc_macro::TokenStream,
  tokens: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
  cacheable::disable_cacheable(tokens)
}

#[proc_macro_attribute]
pub fn cacheable_dyn(
  _args: proc_macro::TokenStream,
  tokens: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
  let input = syn::parse_macro_input!(tokens as syn::Item);

  match input {
    syn::Item::Trait(input) => cacheable_dyn::impl_trait(input),
    syn::Item::Impl(input) => cacheable_dyn::impl_impl(input),
    _ => panic!("expect Trait or Impl"),
  }
}

#[proc_macro_attribute]
pub fn disable_cacheable_dyn(
  _args: proc_macro::TokenStream,
  tokens: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
  cacheable_dyn::disable_cacheable_dyn(tokens)
}

#[proc_macro]
pub fn rspack_version(_tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let version = rspack_version::rspack_version();
  quote::quote! { #version }.into()
}
