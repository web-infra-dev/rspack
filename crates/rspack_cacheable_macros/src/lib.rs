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
  args: proc_macro::TokenStream,
  tokens: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
  cacheable_dyn::cacheable_dyn(args, tokens)
}

#[proc_macro_attribute]
pub fn disable_cacheable_dyn(
  _args: proc_macro::TokenStream,
  tokens: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
  cacheable_dyn::disable_cacheable_dyn(tokens)
}
