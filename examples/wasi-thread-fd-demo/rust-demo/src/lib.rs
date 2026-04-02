use std::{
  fs::File,
  os::fd::AsRawFd,
  sync::OnceLock,
};

const DEMO_PATH: &str = "/tmp/rspack-wasi-thread-fd-demo.txt";

static KEEP_FILES: OnceLock<std::sync::Mutex<Vec<File>>> = OnceLock::new();

#[unsafe(no_mangle)]
pub extern "C" fn demo_open_keep() -> i32 {
  let file = File::open(DEMO_PATH).expect("open demo file");
  let fd = file.as_raw_fd();
  KEEP_FILES
    .get_or_init(|| std::sync::Mutex::new(Vec::new()))
    .lock()
    .expect("lock file store")
    .push(file);
  fd
}
