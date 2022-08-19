#[macro_export]
macro_rules! define_napi_object {
  (
    $(#[derive($($derive:meta),*)])*
    $(#[serde($($serde:meta),*)])*
    $(#[doc=$($s_doc:tt)*])*
    $_vis_struct:vis $name: ident {
      $(
        $(#[doc=$($doc:tt)*])*
        $(#[serde($($serde_attr:meta),*)])*
        $(#[napi($($napi_attr:meta),*)])*
        $_vis_field:vis $field:ident : $type:ty
      ), *
    }
 ) => {
    #[cfg(feature = "node-api")]
    $(#[derive($($derive),*)])*
    $(#[serde($($serde),*)])*
    $(#[doc=$($s_doc)*])*
    #[napi(object)]
    pub struct $name {
      $(
        $(#[doc=$($doc)*])*
        $(#[serde($($serde_attr),*)])*
        $(#[napi($($napi_attr),*)])*
        pub $field: $type
      ), *
    }

    #[cfg(not(feature = "node-api"))]
    $(#[derive($($derive),*)])*
    $(#[serde($($serde),*)])*
    $(#[doc=$($s_doc)*])*
    pub struct $name {
      $(
        $(#[doc=$($doc)*])*
        $(#[serde($($serde_attr),*)])*
        pub $field: $type
      ), *
    }
  };

  ($($rest:tt)*) => {
    compile_error!(r#"Unable to expand macro `define_napi_object`
Please check your syntax. Note that the order of the proc-macros and attribute-macros is important!
Supported proc-macros: `derive` / `serde`.
Supported attribute-macros: `doc`(or, ///) / `serde` / `napi`.

  Example:
  define_napi_object!(
      #[derive(...)]
      #[serde(...)]
      MyObject {
        #[serde(...)]
        #[napi(ts_type = "...")]
        field: Type
      }
  });"#);
  };
}
