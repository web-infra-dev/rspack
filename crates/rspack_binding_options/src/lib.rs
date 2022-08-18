// mod options;
// pub use options::*;
// mod macros;
// pub use macros::*;

#[cfg(feature = "node-api")]
use napi_derive::napi;

macro_rules! generate_struct {
  (
    #[derive($($d_meta:meta),*)]
    #[napi($($napi_meta:meta),*)]
    struct $name:ident {
      $(
        $(
          #[$attr:meta]
        )*
        $field:ident: $field_type:ty,
      )*
    }
  ) => {
    #[cfg(feature = "node-api")]
    use napi_derive::napi;

    #[cfg(feature = "node-api")]
    #[derive($($d_meta), *)]
    #[napi($($napi_meta), *)]
    pub struct $name {
      $(
        $(
          #[$attr]
        )*
        pub $field: $field_type,
      )+
    }
  }
}

generate_struct!(
  #[derive(Debug, Clone)]
  #[napi(object)]
  struct Foo {
    #[napi(ts_type = "any")]
    a: i32,
  }
);
