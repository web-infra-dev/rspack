// Adapted from napi-rs (MIT): https://github.com/napi-rs/napi-rs/pull/3423.
// Replace this backport with napi-rs's helper when upgrading napi-rs.

/// Retain the addon before starting native threads that can outlive a Node environment.
/// This is best effort, matching napi-rs on unsupported platforms and loader failures.
pub fn retain_current_module_for_unload_safety() {
  static RETAIN_MODULE: std::sync::Once = std::sync::Once::new();
  RETAIN_MODULE.call_once(retain_current_module);
}

#[cfg(windows)]
fn retain_current_module() {
  const GET_MODULE_HANDLE_EX_FLAG_PIN: u32 = 0x0000_0001;
  const GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS: u32 = 0x0000_0004;

  #[link(name = "kernel32")]
  unsafe extern "system" {
    fn GetModuleHandleExW(
      flags: u32,
      module_name: *const u16,
      module: *mut *mut std::ffi::c_void,
    ) -> i32;
  }

  let mut module = std::ptr::null_mut();
  // SAFETY: FROM_ADDRESS interprets the second argument as an address inside this addon.
  // PIN retains it until process exit; the returned handle must never be freed.
  unsafe {
    GetModuleHandleExW(
      GET_MODULE_HANDLE_EX_FLAG_PIN | GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS,
      retain_current_module_for_unload_safety as *const () as *const u16,
      &mut module,
    );
  }
}

#[cfg(any(
  target_vendor = "apple",
  target_os = "linux",
  target_os = "android",
  target_os = "freebsd",
  target_os = "dragonfly",
  target_os = "netbsd",
  target_os = "solaris",
  target_os = "illumos"
))]
fn retain_current_module() {
  // glibc before 2.34 keeps the dl* symbols in a separate library.
  #[cfg(any(target_os = "linux", target_os = "android"))]
  #[link(name = "dl")]
  unsafe extern "C" {}

  let anchor = retain_current_module_for_unload_safety as *const () as *const std::ffi::c_void;
  let mut info = std::mem::MaybeUninit::<libc::Dl_info>::zeroed();
  // SAFETY: anchor belongs to this image and info is a valid out-parameter.
  let info = unsafe {
    if libc::dladdr(anchor, info.as_mut_ptr()) == 0 {
      return;
    }
    info.assume_init()
  };
  if info.dli_fname.is_null() {
    return;
  }
  // SAFETY: dladdr provides a NUL-terminated filename owned by the loader. NOLOAD only
  // retains the existing image. Deliberately keep the handle for the process lifetime.
  unsafe {
    libc::dlopen(
      info.dli_fname,
      libc::RTLD_LAZY | libc::RTLD_LOCAL | libc::RTLD_NOLOAD,
    );
  }
}

#[cfg(not(any(
  windows,
  target_vendor = "apple",
  target_os = "linux",
  target_os = "android",
  target_os = "freebsd",
  target_os = "dragonfly",
  target_os = "netbsd",
  target_os = "solaris",
  target_os = "illumos"
)))]
fn retain_current_module() {}
