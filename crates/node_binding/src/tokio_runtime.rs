use once_cell::sync::Lazy;

/// TODO(hyf0): we might need a way to cleanup this resource
static TOKIO_RT: Lazy<tokio::runtime::Runtime> = Lazy::new(init_tokio_runtime);

fn init_tokio_runtime() -> tokio::runtime::Runtime {
  tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    .build()
    .expect("should initial tokio runtime without error")
}

pub fn tokio_rt() -> &'static tokio::runtime::Runtime {
  &TOKIO_RT
}
