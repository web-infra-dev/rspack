use once_cell::sync::Lazy;

/// TODO(hyf0): we might need a way to cleanup this resource
static TOKIO_RT: Lazy<tokio::runtime::Runtime> = Lazy::new(init_tokio_runtime);

fn init_tokio_runtime() -> tokio::runtime::Runtime {
  //
  tokio::runtime::Builder::new_multi_thread()
    // See https://github.com/web-infra-dev/rspack/pull/183
    // 6mb+
    .thread_stack_size(6777216)
    .build()
    .expect("should initial tokio runtime without error")
}

pub fn tokio_rt() -> &'static tokio::runtime::Runtime {
  &TOKIO_RT
}
