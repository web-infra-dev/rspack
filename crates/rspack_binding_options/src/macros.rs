#[macro_export]
macro_rules! generate_struct {
  (
    $(#[doc=$($s_doc:tt)*])*
    $name: ident {
      $(
        $(#[doc=$($doc:tt)*])*
        $field: ident: $type: ty,
        #[napi($($napi_attr:meta), *)]
      ),*
    }
 ) => {
    $(#[doc=$($s_doc)*])*
    pub struct $name {
      $(
        $(#[doc=$($doc)*])*
        pub $field: $type
      ),*
    }
  };

  (
    $(#[doc=$($s_doc:tt)*])*
    $name: ident {
      $(
        $(#[doc=$($doc:tt)*])*
        $field: ident: $type: ty,
        #[napi($($napi_attr:meta), *)],
        #[serde($($serde_attr:meta), *)],
      ),*
    }
  ) => {
    $(#[doc=$($s_doc)*])*
    pub struct $name {
      $(
        $(#[doc=$($doc)*])*
        pub $field: $type
      ),*
    }
  };

  (
    $(#[doc=$($s_doc:tt)*])*
    $name: ident {
      $(
        $(#[doc=$($doc:tt)*])*
        $field: ident: $type: ty,
        #[serde($($serde_attr:meta), *)],
      ),*
    }
  ) => {
    $(#[doc=$($s_doc)*])*
    pub struct $name {
      $(
        $(#[doc=$($doc)*])*
        pub $field: $type
      ),*
    }
  };

  (
    $(#[doc=$($s_doc:tt)*])*
    $name: ident, {
      $(
        $(#[doc=$($doc:tt)*])*
        $field: ident: $type: ty
      ),*
    }
  ) => {
    // $(#[doc=$($s_doc)*])*
    pub struct $name {
      $(
        $(#[doc=$($doc)*])*
        pub $field: $type
      ),*
    }
  };
}
