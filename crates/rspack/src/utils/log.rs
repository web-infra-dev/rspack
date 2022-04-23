static mut IS_TRACING_ENABLED: bool = false;

pub fn enable_tracing_by_env() {
  let is_enable_tracing = std::env::var("TRACE").is_ok();
  if is_enable_tracing {
    unsafe {
      if !IS_TRACING_ENABLED {
        IS_TRACING_ENABLED = true;
        use tracing_chrome::ChromeLayerBuilder;
        use tracing_subscriber::{fmt, prelude::*, registry::Registry};

        let (chrome_layer, _guard) = ChromeLayerBuilder::new().build();
        tracing_subscriber::registry()
          .with(chrome_layer)
          .with(fmt::layer())
          .init();
      }
    }
  }
}
