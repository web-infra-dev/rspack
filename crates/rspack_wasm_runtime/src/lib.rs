mod runtime_impl;

const RSPACK_WASM_RUNTIME_ABI_VERSION: u32 = 1;

#[unsafe(no_mangle)]
pub extern "C" fn rspack_wasm_runtime_abi_version() -> u32 {
  RSPACK_WASM_RUNTIME_ABI_VERSION
}

#[unsafe(no_mangle)]
pub extern "C" fn rspack_wasm_runtime_last_error() -> *const std::ffi::c_char {
  runtime_impl::last_error()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rspack_wasm_runtime_prepare_module(
  bytes: runtime_impl::BytesView,
  cache: *mut *mut std::ffi::c_void,
) -> bool {
  unsafe { runtime_impl::prepare_module(bytes, cache) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rspack_wasm_runtime_clone_cache(
  cache: *mut std::ffi::c_void,
  cloned: *mut *mut std::ffi::c_void,
) -> bool {
  unsafe { runtime_impl::clone_cache(cache, cloned) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rspack_wasm_runtime_load_cache(
  path: runtime_impl::BytesView,
  cache: *mut *mut std::ffi::c_void,
) -> bool {
  unsafe { runtime_impl::load_cache(path, cache) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rspack_wasm_runtime_store_cache(
  path: runtime_impl::BytesView,
  cache: *mut std::ffi::c_void,
) -> bool {
  unsafe { runtime_impl::store_cache(path, cache) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rspack_wasm_runtime_destroy_cache(cache: *mut std::ffi::c_void) {
  unsafe { runtime_impl::destroy_cache(cache) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rspack_wasm_runtime_init(
  name: runtime_impl::BytesView,
  imports: *const runtime_impl::Import,
  imports_len: usize,
  envs: *const runtime_impl::Env,
  envs_len: usize,
  module: runtime_impl::Module,
  instance: *mut *mut std::ffi::c_void,
) -> bool {
  unsafe { runtime_impl::init(name, imports, imports_len, envs, envs_len, module, instance) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rspack_wasm_runtime_instance_transform(
  instance: *mut std::ffi::c_void,
  program_ptr: u32,
  program_len: u32,
  unresolved_mark: u32,
  should_enable_comments_proxy: u32,
  result: *mut u32,
) -> bool {
  unsafe {
    runtime_impl::instance_transform(
      instance,
      program_ptr,
      program_len,
      unresolved_mark,
      should_enable_comments_proxy,
      result,
    )
  }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rspack_wasm_runtime_instance_read_buf(
  instance: *mut std::ffi::c_void,
  ptr: u32,
  buf: *mut u8,
  len: usize,
) -> bool {
  unsafe { runtime_impl::instance_read_buf(instance, ptr, buf, len) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rspack_wasm_runtime_instance_write_buf(
  instance: *mut std::ffi::c_void,
  ptr: u32,
  buf: *const u8,
  len: usize,
) -> bool {
  unsafe { runtime_impl::instance_write_buf(instance, ptr, buf, len) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rspack_wasm_runtime_instance_alloc(
  instance: *mut std::ffi::c_void,
  size: u32,
  result: *mut u32,
) -> bool {
  unsafe { runtime_impl::instance_alloc(instance, size, result) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rspack_wasm_runtime_instance_free(
  instance: *mut std::ffi::c_void,
  ptr: u32,
  size: u32,
  result: *mut u32,
) -> bool {
  unsafe { runtime_impl::instance_free(instance, ptr, size, result) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rspack_wasm_runtime_instance_cache(
  instance: *mut std::ffi::c_void,
  cache: *mut *mut std::ffi::c_void,
) -> bool {
  unsafe { runtime_impl::instance_cache(instance, cache) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rspack_wasm_runtime_destroy_instance(instance: *mut std::ffi::c_void) {
  unsafe { runtime_impl::destroy_instance(instance) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rspack_wasm_runtime_caller_read_buf(
  caller: *mut std::ffi::c_void,
  ptr: u32,
  buf: *mut u8,
  len: usize,
) -> bool {
  unsafe { runtime_impl::caller_read_buf(caller, ptr, buf, len) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rspack_wasm_runtime_caller_write_buf(
  caller: *mut std::ffi::c_void,
  ptr: u32,
  buf: *const u8,
  len: usize,
) -> bool {
  unsafe { runtime_impl::caller_write_buf(caller, ptr, buf, len) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rspack_wasm_runtime_caller_alloc(
  caller: *mut std::ffi::c_void,
  size: u32,
  result: *mut u32,
) -> bool {
  unsafe { runtime_impl::caller_alloc(caller, size, result) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rspack_wasm_runtime_caller_free(
  caller: *mut std::ffi::c_void,
  ptr: u32,
  size: u32,
  result: *mut u32,
) -> bool {
  unsafe { runtime_impl::caller_free(caller, ptr, size, result) }
}
