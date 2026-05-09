use std::sync::Once;

static RAYON_FOR_CODSPEED: Once = Once::new();

/// Make Rayon use the benchmark thread as worker 0 so CodSpeed attributes
/// rayon work to the measured parent function.
pub fn configure_rayon_current_thread_for_codspeed() {
  RAYON_FOR_CODSPEED.call_once(|| {
    rayon::ThreadPoolBuilder::new()
      .use_current_thread()
      .build_global()
      .expect("rayon global thread pool should be configured before rayon is used");
  });
}
