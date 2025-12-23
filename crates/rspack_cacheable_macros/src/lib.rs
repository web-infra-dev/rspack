mod cacheable;
mod cacheable_dyn;

#[proc_macro_attribute]
pub fn enable_cacheable(
  args: proc_macro::TokenStream,
  tokens: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
  cacheable::cacheable(args, tokens)
}

#[proc_macro_attribute]
pub fn disable_cacheable(
  args: proc_macro::TokenStream,
  tokens: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
  cacheable::disable_cacheable(args, tokens)
}

#[proc_macro_attribute]
pub fn enable_cacheable_dyn(
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
