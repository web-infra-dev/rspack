pub fn install_panic_handler() {
  use std::{fs::File, io::Write, os::fd::FromRawFd};

  std::panic::set_hook(Box::new(|info| {
    let msg = info.to_string();
    let mut file = unsafe { File::from_raw_fd(2) };
    file.write_all(msg.as_bytes()).unwrap();
  }));
}
