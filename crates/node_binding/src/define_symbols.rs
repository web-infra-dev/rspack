#[macro_export]
macro_rules! define_symbols {
  (
    $(
      $(#[$meta:meta])*
      $cell:ident => $name:expr
    ),* $(,)?
  ) => {
    thread_local! {
      $(
        $(#[$meta])*
        pub(crate) static $cell: ::once_cell::unsync::OnceCell<::rspack_napi::OneShotRef> = Default::default();
      )*
    }

    pub(super) fn export_symbols(mut exports: ::napi::bindgen_prelude::Object, env: ::napi::Env) -> ::napi::Result<()> {
      $(
        let symbol = ::rspack_napi::OneShotRef::new(env.raw(), env.create_symbol(Some($name))?)?;
        exports.set_named_property($name, &symbol)?;
        $cell.with(|once_cell| {
          once_cell.get_or_init(move || symbol);
        });
      )*
      Ok(())
    }
  };
}
