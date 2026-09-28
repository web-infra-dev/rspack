mod ast_object;
mod hook;
mod javascript_parser_plugin_hooks;
mod merge;
mod plugin;
mod rspack_hash;
mod runtime_module;
mod source_map_config;
mod string_enum;

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

#[proc_macro_attribute]
pub fn implemented_javascript_parser_hooks(
  args: proc_macro::TokenStream,
  tokens: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
  javascript_parser_plugin_hooks::expand(args, tokens)
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

/// Derives `as_str` and `From<&str>` for a string enum.
///
/// Variant names use `snake_case` by default. Use `#[string_enum(rename_all = "...")]` on the enum
/// to select a serde-style rename rule, or `#[string_enum(rename = "...")]` on a variant to
/// override its value. Mark exactly one variant with `#[string_enum(fallback)]` for unknown
/// strings. The fallback may be a unit variant or a newtype variant that can store the unknown
/// string.
#[proc_macro_derive(StringEnum, attributes(string_enum))]
pub fn string_enum_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let input = syn::parse_macro_input!(input as syn::DeriveInput);
  match string_enum::expand(input) {
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

#[proc_macro_derive(RspackHash, attributes(rspack_hash))]
pub fn rspack_hash_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let input = syn::parse_macro_input!(input as syn::DeriveInput);
  let output = rspack_hash::expand_rspack_hash_derive(input);
  match output {
    syn::Result::Ok(tt) => tt,
    syn::Result::Err(err) => err.to_compile_error(),
  }
  .into()
}

#[proc_macro_derive(AstObject, attributes(ast_object))]
pub fn ast_object_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let input = syn::parse_macro_input!(input as syn::DeriveInput);
  let output = ast_object::expand_ast_object_derive(input);
  match output {
    syn::Result::Ok(tt) => tt,
    syn::Result::Err(err) => err.to_compile_error(),
  }
  .into()
}
