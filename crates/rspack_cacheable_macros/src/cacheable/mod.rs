mod args;
mod r#impl;
mod impl_disable;
mod impl_with;

use args::CacheableArgs;
use r#impl::impl_cacheable;
use impl_disable::impl_disable_cacheable;
use impl_with::impl_cacheable_with;

pub fn cacheable(
  args: proc_macro::TokenStream,
  tokens: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
  let args = syn::parse_macro_input!(args as CacheableArgs);
  if args.with.is_some() {
    impl_cacheable_with(tokens, args)
  } else {
    impl_cacheable(tokens, args)
  }
}

pub fn disable_cacheable(
  args: proc_macro::TokenStream,
  tokens: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
  let args = syn::parse_macro_input!(args as CacheableArgs);
  impl_disable_cacheable(tokens, args)
}
