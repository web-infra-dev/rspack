use std::sync::Once;

static RAYON_FOR_CODSPEED: Once = Once::new();

const DEFAULT_WORKER_THREAD_STACK_SIZE: usize = 2 * 1024 * 1024;
const WORKER_THREAD_STACK_SIZE: usize = DEFAULT_WORKER_THREAD_STACK_SIZE + 20_000_000;

/// Make Rayon use the benchmark thread as worker 0 so CodSpeed attributes
/// rayon work to the measured parent function.
pub fn configure_rayon_current_thread_for_codspeed() {
  RAYON_FOR_CODSPEED.call_once(|| {
    rayon::ThreadPoolBuilder::new()
      .use_current_thread()
      .stack_size(WORKER_THREAD_STACK_SIZE)
      .build_global()
      .expect("rayon global thread pool should be configured before rayon is used");
  });
}
