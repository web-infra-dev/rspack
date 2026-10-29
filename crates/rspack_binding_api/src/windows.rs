use windows_sys::Win32::System::LibraryLoader::{
  GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS, GET_MODULE_HANDLE_EX_FLAG_PIN, GetModuleHandleExW,
};

pub(crate) fn pin_module() -> napi::Result<()> {
  let mut module = std::ptr::null_mut();
  // Rayon workers and background runtime shutdown can outlive the last Node worker.
  // Keep their code mapped even after Node releases its last handle to this addon.
  // SAFETY: this function's address belongs to the loaded addon. FROM_ADDRESS treats
  // it as an address rather than a string, and `module` is a valid output pointer.
  let success = unsafe {
    GetModuleHandleExW(
      GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_PIN,
      pin_module as *const () as *const u16,
      &mut module,
    )
  };
  if success == 0 {
    return Err(napi::Error::from_reason(format!(
      "Failed to keep the Rspack native binding loaded: {}",
      std::io::Error::last_os_error()
    )));
  }
  Ok(())
}
