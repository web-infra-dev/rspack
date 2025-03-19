#![feature(try_find)]

mod field_names;
mod tagged_union;

#[proc_macro_attribute]
pub fn field_names(
  args: proc_macro::TokenStream,
  tokens: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
  field_names::field_names(args, tokens)
}

///
/// ## Difference from `napi::Either`
///
/// `napi::Either` is designed for different type of values.
/// Under the hood, the type is differentiated by `napi_typeof`.
///
/// `tagged_union` is designed for different variants of the same type.
///
/// ## Example
/// ```ignore
/// #[tagged_union]
/// enum Foo {
///   V1(V1),
///   #[napi(ts_type = "Record<string, string>")]
///   V2(V2)
/// }
///
/// ⬇️⬇️⬇️
///
/// #[napi(js_name = "Foo")]
/// struct __rspack_napi_macros_Foo {
///   #[napi(ts_type = "FooTypes")]
///   r#type: String,
///   V1: Option<V1>,
///   #[napi(ts_type = "Record<string, string>")]
///   V2: Option<V2>
/// }
///
/// #[napi(string_enum)]
/// enum FooTypes {
///   V1,
///   V2
/// }
///
/// impl From<__rspack_napi_macros_Foo> for Foo {
///   fn from(value: __rspack_napi_macros_Foo) -> Self {
///      // ..
///   }
/// }
///
/// impl FromNapiValue for Foo {
///    fn from_napi_value(env, val) -> Self {
///       let item: __rspack_napi_macros_Foo  = val.into();
///       item.into()
///    }
/// }
/// ```
#[proc_macro_attribute]
pub fn tagged_union(
  _args: proc_macro::TokenStream,
  tokens: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
  tagged_union::expand(tokens.into()).into()
}
