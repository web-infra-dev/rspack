use once_cell::sync::OnceCell;

pub fn init_rayon_thread_poll() {
  static INSTANCE: OnceCell<()> = OnceCell::new();
  INSTANCE.get_or_init(|| {
    rayon::ThreadPoolBuilder::new()
      .stack_size(16777216)
      .build_global()
      .unwrap()
  });
}
