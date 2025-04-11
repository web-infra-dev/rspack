mod args;
mod r#impl;
mod impl_as;
mod impl_disable;
mod impl_with;

use args::CacheableArgs;
use impl_as::impl_cacheable_as;
use impl_disable::impl_disable_cacheable;
use impl_with::impl_cacheable_with;
use r#impl::impl_cacheable;

pub fn cacheable(
  args: proc_macro::TokenStream,
  tokens: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
  let args = syn::parse_macro_input!(args as CacheableArgs);
  if args.with.is_some() {
    impl_cacheable_with(tokens, args)
  } else if args.r#as.is_some() {
    impl_cacheable_as(tokens, args)
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
