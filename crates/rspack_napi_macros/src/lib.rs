#![feature(try_find)]

mod getters;
mod tagged_union;

#[proc_macro_attribute]
pub fn getters(
  args: proc_macro::TokenStream,
  tokens: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
  getters::getters(args, tokens)
}

#[proc_macro_attribute]
pub fn tagged_union(
  _args: proc_macro::TokenStream,
  tokens: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
  tagged_union::expand(tokens.into()).into()
}
