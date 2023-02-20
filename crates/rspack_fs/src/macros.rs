macro_rules! cfg_async {
  ($($item:item)*) => {
      $( #[cfg(feature = "async")] $item )*
  }
}

macro_rules! cfg_native {
  ($($item:item)*) => {
      $( #[cfg(feature = "native")] $item )*
  }
}
