mod args;
mod impl_disable;
mod impl_impl;
mod impl_trait;

use args::DynArgs;
use impl_disable::impl_disable;
use impl_impl::impl_impl;
use impl_trait::impl_trait;

pub fn cacheable_dyn(
  args: proc_macro::TokenStream,
  tokens: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
  let input = syn::parse_macro_input!(tokens as syn::Item);
  let args = syn::parse_macro_input!(args as DynArgs);

  match input {
    syn::Item::Trait(input) => impl_trait(input, args),
    syn::Item::Impl(input) => impl_impl(input, args),
    _ => panic!("expect Trait or Impl"),
  }
}

pub fn disable_cacheable_dyn(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
  impl_disable(tokens)
}
