use std::{
  any::Any,
  ffi::{CStr, c_char, c_void},
  fmt,
  marker::PhantomData,
  path::{Path, PathBuf},
  ptr, slice,
};

use anyhow::{Context, bail};
use libloading::Library;
use once_cell::sync::OnceCell;
use swc_plugin_runner::runtime;

/// Identifier for bytecode cache stored in local filesystem.
///
/// This MUST be updated when bump up wasmtime.
const MODULE_SERIALIZATION_IDENTIFIER: &str = concat!("wasmtime", "-", "v36");
const RSPACK_WASM_RUNTIME_ABI_VERSION: u32 = 1;
const RSPACK_WASM_RUNTIME_LIBRARY_PATH: &str = "RSPACK_WASM_RUNTIME_LIBRARY_PATH";

static WASM_RUNTIME_LIBRARY_PATH: OnceCell<PathBuf> = OnceCell::new();
static WASM_RUNTIME: OnceCell<DynamicWasmtimeRuntime> = OnceCell::new();

#[repr(C)]
#[derive(Clone, Copy)]
struct BytesView {
  ptr: *const u8,
  len: usize,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Import {
  name: BytesView,
  params: u8,
  results: u8,
  data: *mut c_void,
  callback: ImportCallback,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Env {
  key: BytesView,
  value: BytesView,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Module {
  kind: u8,
  bytes: BytesView,
  cache: *mut c_void,
}

type ImportCallback = unsafe extern "C" fn(
  data: *mut c_void,
  caller: *mut c_void,
  input: *const i32,
  input_len: usize,
  output: *mut i32,
  output_len: usize,
);

type AbiVersion = unsafe extern "C" fn() -> u32;
type LastError = unsafe extern "C" fn() -> *const c_char;
type PrepareModule = unsafe extern "C" fn(BytesView, *mut *mut c_void) -> bool;
type CloneCache = unsafe extern "C" fn(*mut c_void, *mut *mut c_void) -> bool;
type LoadCache = unsafe extern "C" fn(BytesView, *mut *mut c_void) -> bool;
type StoreCache = unsafe extern "C" fn(BytesView, *mut c_void) -> bool;
type DestroyCache = unsafe extern "C" fn(*mut c_void);
type Init = unsafe extern "C" fn(
  BytesView,
  *const Import,
  usize,
  *const Env,
  usize,
  Module,
  *mut *mut c_void,
) -> bool;
type InstanceTransform = unsafe extern "C" fn(*mut c_void, u32, u32, u32, u32, *mut u32) -> bool;
type InstanceReadBuf = unsafe extern "C" fn(*mut c_void, u32, *mut u8, usize) -> bool;
type InstanceWriteBuf = unsafe extern "C" fn(*mut c_void, u32, *const u8, usize) -> bool;
type InstanceAlloc = unsafe extern "C" fn(*mut c_void, u32, *mut u32) -> bool;
type InstanceFree = unsafe extern "C" fn(*mut c_void, u32, u32, *mut u32) -> bool;
type InstanceCache = unsafe extern "C" fn(*mut c_void, *mut *mut c_void) -> bool;
type DestroyInstance = unsafe extern "C" fn(*mut c_void);
type CallerReadBuf = unsafe extern "C" fn(*mut c_void, u32, *mut u8, usize) -> bool;
type CallerWriteBuf = unsafe extern "C" fn(*mut c_void, u32, *const u8, usize) -> bool;
type CallerAlloc = unsafe extern "C" fn(*mut c_void, u32, *mut u32) -> bool;
type CallerFree = unsafe extern "C" fn(*mut c_void, u32, u32, *mut u32) -> bool;

#[derive(Clone, Copy, Debug)]
pub struct WasmtimeRuntime;

struct DynamicWasmtimeRuntime {
  _library: Library,
  last_error: LastError,
  prepare_module: PrepareModule,
  clone_cache: CloneCache,
  load_cache: LoadCache,
  store_cache: StoreCache,
  destroy_cache: DestroyCache,
  init: Init,
  instance_transform: InstanceTransform,
  instance_read_buf: InstanceReadBuf,
  instance_write_buf: InstanceWriteBuf,
  instance_alloc: InstanceAlloc,
  instance_free: InstanceFree,
  instance_cache: InstanceCache,
  destroy_instance: DestroyInstance,
  caller_read_buf: CallerReadBuf,
  caller_write_buf: CallerWriteBuf,
  caller_alloc: CallerAlloc,
  caller_free: CallerFree,
}

impl fmt::Debug for DynamicWasmtimeRuntime {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("DynamicWasmtimeRuntime")
      .finish_non_exhaustive()
  }
}

struct DynamicModuleCache {
  handle: *mut c_void,
  runtime: &'static DynamicWasmtimeRuntime,
}

unsafe impl Send for DynamicModuleCache {}
unsafe impl Sync for DynamicModuleCache {}

impl DynamicModuleCache {
  fn into_handle(mut self: Box<Self>) -> *mut c_void {
    let handle = self.handle;
    self.handle = ptr::null_mut();
    handle
  }
}

impl Drop for DynamicModuleCache {
  fn drop(&mut self) {
    if !self.handle.is_null() {
      unsafe {
        (self.runtime.destroy_cache)(self.handle);
      }
      self.handle = ptr::null_mut();
    }
  }
}

struct ImportData {
  func: runtime::Func,
  runtime: &'static DynamicWasmtimeRuntime,
}

struct DynamicInstance {
  handle: *mut c_void,
  runtime: &'static DynamicWasmtimeRuntime,
  _imports: Vec<Box<ImportData>>,
}

unsafe impl Send for DynamicInstance {}
unsafe impl Sync for DynamicInstance {}

impl Drop for DynamicInstance {
  fn drop(&mut self) {
    if !self.handle.is_null() {
      unsafe {
        (self.runtime.destroy_instance)(self.handle);
      }
      self.handle = ptr::null_mut();
    }
  }
}

enum DynamicCallerTarget {
  Instance(*mut c_void),
  ImportCaller(*mut c_void),
}

struct DynamicCaller<'a> {
  runtime: &'static DynamicWasmtimeRuntime,
  target: DynamicCallerTarget,
  _marker: PhantomData<&'a mut ()>,
}

pub fn set_wasm_runtime_library_path(path: impl Into<PathBuf>) -> anyhow::Result<()> {
  let path = path.into();
  if let Some(existing) = WASM_RUNTIME_LIBRARY_PATH.get() {
    if existing != &path {
      bail!(
        "wasm runtime library path has already been set to {}, cannot set it to {}",
        existing.display(),
        path.display()
      );
    }
    return Ok(());
  }

  WASM_RUNTIME_LIBRARY_PATH.set(path).map_err(|path| {
    anyhow::format_err!(
      "failed to set wasm runtime library path to {}",
      path.display()
    )
  })
}

fn wasm_runtime_library_path() -> anyhow::Result<PathBuf> {
  if let Some(path) = WASM_RUNTIME_LIBRARY_PATH.get() {
    return Ok(path.clone());
  }

  if let Some(path) = std::env::var_os(RSPACK_WASM_RUNTIME_LIBRARY_PATH) {
    return Ok(path.into());
  }

  bail!(
    "failed to locate @rspack/wasm-runtime. Install @rspack/wasm-runtime or set {RSPACK_WASM_RUNTIME_LIBRARY_PATH} to the wasm runtime dynamic library path."
  )
}

fn symbol_name(name: &[u8]) -> String {
  String::from_utf8_lossy(name.strip_suffix(b"\0").unwrap_or(name)).into_owned()
}

unsafe fn load_symbol<T: Copy>(library: &Library, name: &'static [u8]) -> anyhow::Result<T> {
  let symbol = unsafe { library.get::<T>(name) }
    .with_context(|| format!("failed to load {} symbol", symbol_name(name)))?;
  Ok(*symbol)
}

fn load_wasm_runtime() -> anyhow::Result<DynamicWasmtimeRuntime> {
  let path = wasm_runtime_library_path()?;
  let library = unsafe { Library::new(&path) }
    .with_context(|| format!("failed to load wasm runtime library {}", path.display()))?;

  unsafe {
    let abi_version = load_symbol::<AbiVersion>(&library, b"rspack_wasm_runtime_abi_version\0")?;
    let abi_version = abi_version();
    if abi_version != RSPACK_WASM_RUNTIME_ABI_VERSION {
      bail!(
        "incompatible @rspack/wasm-runtime ABI version: expected {}, got {}",
        RSPACK_WASM_RUNTIME_ABI_VERSION,
        abi_version
      );
    }

    Ok(DynamicWasmtimeRuntime {
      last_error: load_symbol::<LastError>(&library, b"rspack_wasm_runtime_last_error\0")?,
      prepare_module: load_symbol::<PrepareModule>(
        &library,
        b"rspack_wasm_runtime_prepare_module\0",
      )?,
      clone_cache: load_symbol::<CloneCache>(&library, b"rspack_wasm_runtime_clone_cache\0")?,
      load_cache: load_symbol::<LoadCache>(&library, b"rspack_wasm_runtime_load_cache\0")?,
      store_cache: load_symbol::<StoreCache>(&library, b"rspack_wasm_runtime_store_cache\0")?,
      destroy_cache: load_symbol::<DestroyCache>(&library, b"rspack_wasm_runtime_destroy_cache\0")?,
      init: load_symbol::<Init>(&library, b"rspack_wasm_runtime_init\0")?,
      instance_transform: load_symbol::<InstanceTransform>(
        &library,
        b"rspack_wasm_runtime_instance_transform\0",
      )?,
      instance_read_buf: load_symbol::<InstanceReadBuf>(
        &library,
        b"rspack_wasm_runtime_instance_read_buf\0",
      )?,
      instance_write_buf: load_symbol::<InstanceWriteBuf>(
        &library,
        b"rspack_wasm_runtime_instance_write_buf\0",
      )?,
      instance_alloc: load_symbol::<InstanceAlloc>(
        &library,
        b"rspack_wasm_runtime_instance_alloc\0",
      )?,
      instance_free: load_symbol::<InstanceFree>(&library, b"rspack_wasm_runtime_instance_free\0")?,
      instance_cache: load_symbol::<InstanceCache>(
        &library,
        b"rspack_wasm_runtime_instance_cache\0",
      )?,
      destroy_instance: load_symbol::<DestroyInstance>(
        &library,
        b"rspack_wasm_runtime_destroy_instance\0",
      )?,
      caller_read_buf: load_symbol::<CallerReadBuf>(
        &library,
        b"rspack_wasm_runtime_caller_read_buf\0",
      )?,
      caller_write_buf: load_symbol::<CallerWriteBuf>(
        &library,
        b"rspack_wasm_runtime_caller_write_buf\0",
      )?,
      caller_alloc: load_symbol::<CallerAlloc>(&library, b"rspack_wasm_runtime_caller_alloc\0")?,
      caller_free: load_symbol::<CallerFree>(&library, b"rspack_wasm_runtime_caller_free\0")?,
      _library: library,
    })
  }
}

fn wasm_runtime() -> anyhow::Result<&'static DynamicWasmtimeRuntime> {
  WASM_RUNTIME.get_or_try_init(load_wasm_runtime)
}

fn bytes_view(bytes: &[u8]) -> BytesView {
  BytesView {
    ptr: bytes.as_ptr(),
    len: bytes.len(),
  }
}

fn path_view(path: &Path) -> (String, BytesView) {
  let path = path.to_string_lossy().into_owned();
  let view = bytes_view(path.as_bytes());
  (path, view)
}

fn check_cache<'a>(
  runtime: &'static DynamicWasmtimeRuntime,
  cache: &'a runtime::ModuleCache,
) -> anyhow::Result<&'a DynamicModuleCache> {
  let cache = cache
    .0
    .downcast_ref::<DynamicModuleCache>()
    .context("wasm runtime module cache was created by an incompatible runtime")?;
  if !ptr::addr_eq(cache.runtime, runtime) {
    bail!("wasm runtime module cache was created by another runtime");
  }
  if cache.handle.is_null() {
    bail!("wasm runtime module cache handle is null");
  }
  Ok(cache)
}

impl DynamicWasmtimeRuntime {
  fn error(&self, context: &str) -> anyhow::Error {
    let last_error = unsafe { (self.last_error)() };
    if last_error.is_null() {
      anyhow::format_err!("{context}")
    } else {
      let last_error = unsafe { CStr::from_ptr(last_error) }.to_string_lossy();
      anyhow::format_err!("{context}: {last_error}")
    }
  }

  fn ensure(&self, ok: bool, context: &str) -> anyhow::Result<()> {
    if ok { Ok(()) } else { Err(self.error(context)) }
  }

  fn call_u32(&self, call: impl FnOnce(*mut u32) -> bool, context: &str) -> anyhow::Result<u32> {
    let mut value = 0;
    self.ensure(call(&mut value), context)?;
    Ok(value)
  }

  fn call_handle(
    &self,
    call: impl FnOnce(*mut *mut c_void) -> bool,
    context: &str,
  ) -> anyhow::Result<*mut c_void> {
    let mut handle = ptr::null_mut();
    self.ensure(call(&mut handle), context)?;
    Ok(handle)
  }
}

impl runtime::Runtime for WasmtimeRuntime {
  fn identifier(&self) -> &'static str {
    MODULE_SERIALIZATION_IDENTIFIER
  }

  fn prepare_module(&self, bytes: &[u8]) -> anyhow::Result<runtime::ModuleCache> {
    let runtime = wasm_runtime()?;
    let handle = runtime.call_handle(
      |cache| unsafe { (runtime.prepare_module)(bytes_view(bytes), cache) },
      "failed to prepare wasm module",
    )?;
    Ok(runtime::ModuleCache(Box::new(DynamicModuleCache {
      handle,
      runtime,
    })))
  }

  fn clone_cache(&self, cache: &runtime::ModuleCache) -> Option<runtime::ModuleCache> {
    let runtime = wasm_runtime().ok()?;
    let cache = check_cache(runtime, cache).ok()?;
    let handle = runtime
      .call_handle(
        |cloned| unsafe { (runtime.clone_cache)(cache.handle, cloned) },
        "failed to clone wasm module cache",
      )
      .ok()?;
    Some(runtime::ModuleCache(Box::new(DynamicModuleCache {
      handle,
      runtime,
    })))
  }

  unsafe fn load_cache(&self, path: &Path) -> Option<runtime::ModuleCache> {
    let runtime = wasm_runtime().ok()?;
    let (_path, path_view) = path_view(path);
    let handle = runtime
      .call_handle(
        |cache| unsafe { (runtime.load_cache)(path_view, cache) },
        "failed to load wasm module cache",
      )
      .ok()?;
    if handle.is_null() {
      None
    } else {
      Some(runtime::ModuleCache(Box::new(DynamicModuleCache {
        handle,
        runtime,
      })))
    }
  }

  fn store_cache(&self, path: &Path, cache: &runtime::ModuleCache) -> anyhow::Result<()> {
    let runtime = wasm_runtime()?;
    let cache = check_cache(runtime, cache)?;
    let (_path, path_view) = path_view(path);
    runtime.ensure(
      unsafe { (runtime.store_cache)(path_view, cache.handle) },
      "failed to store wasm module cache",
    )
  }

  fn init(
    &self,
    name: &str,
    imports: Vec<(String, runtime::Func)>,
    envs: Vec<(String, String)>,
    module: runtime::Module,
  ) -> anyhow::Result<Box<dyn runtime::Instance>> {
    let runtime = wasm_runtime()?;
    let mut import_names = Vec::with_capacity(imports.len());
    let mut import_data = Vec::with_capacity(imports.len());
    for (name, func) in imports {
      import_names.push(name);
      import_data.push(Box::new(ImportData { func, runtime }));
    }

    let mut ffi_imports = Vec::with_capacity(import_data.len());
    for (name, data) in import_names.iter().zip(import_data.iter_mut()) {
      ffi_imports.push(Import {
        name: bytes_view(name.as_bytes()),
        params: data.func.sign.0,
        results: data.func.sign.1,
        data: (&mut **data as *mut ImportData).cast(),
        callback: call_import,
      });
    }

    let ffi_envs = envs
      .iter()
      .map(|(key, value)| Env {
        key: bytes_view(key.as_bytes()),
        value: bytes_view(value.as_bytes()),
      })
      .collect::<Vec<_>>();

    let handle = match module {
      runtime::Module::Bytes(bytes) => {
        let module = Module {
          kind: 0,
          bytes: bytes_view(&bytes),
          cache: ptr::null_mut(),
        };
        runtime.call_handle(
          |instance| unsafe {
            (runtime.init)(
              bytes_view(name.as_bytes()),
              ffi_imports.as_ptr(),
              ffi_imports.len(),
              ffi_envs.as_ptr(),
              ffi_envs.len(),
              module,
              instance,
            )
          },
          "failed to initialize wasm module",
        )?
      }
      runtime::Module::Cache(cache) => {
        let cache =
          cache
            .0
            .downcast::<DynamicModuleCache>()
            .map_err(|_: Box<dyn Any + Send + Sync>| {
              anyhow::format_err!(
                "wasm runtime module cache was created by an incompatible runtime"
              )
            })?;
        if !ptr::addr_eq(cache.runtime, runtime) {
          bail!("wasm runtime module cache was created by another runtime");
        }
        let cache = cache.into_handle();
        let module = Module {
          kind: 1,
          bytes: BytesView {
            ptr: ptr::null(),
            len: 0,
          },
          cache,
        };
        runtime.call_handle(
          |instance| unsafe {
            (runtime.init)(
              bytes_view(name.as_bytes()),
              ffi_imports.as_ptr(),
              ffi_imports.len(),
              ffi_envs.as_ptr(),
              ffi_envs.len(),
              module,
              instance,
            )
          },
          "failed to initialize wasm module",
        )?
      }
    };

    Ok(Box::new(DynamicInstance {
      handle,
      runtime,
      _imports: import_data,
    }))
  }
}

impl runtime::Instance for DynamicInstance {
  fn transform(
    &mut self,
    program_ptr: u32,
    program_len: u32,
    unresolved_mark: u32,
    should_enable_comments_proxy: u32,
  ) -> anyhow::Result<u32> {
    self.runtime.call_u32(
      |result| unsafe {
        (self.runtime.instance_transform)(
          self.handle,
          program_ptr,
          program_len,
          unresolved_mark,
          should_enable_comments_proxy,
          result,
        )
      },
      "failed to transform with wasm plugin",
    )
  }

  fn caller(&mut self) -> anyhow::Result<Box<dyn runtime::Caller<'_> + '_>> {
    Ok(Box::new(DynamicCaller {
      runtime: self.runtime,
      target: DynamicCallerTarget::Instance(self.handle),
      _marker: PhantomData,
    }))
  }

  fn cache(&self) -> Option<runtime::ModuleCache> {
    let handle = self
      .runtime
      .call_handle(
        |cache| unsafe { (self.runtime.instance_cache)(self.handle, cache) },
        "failed to export wasm module cache",
      )
      .ok()?;
    Some(runtime::ModuleCache(Box::new(DynamicModuleCache {
      handle,
      runtime: self.runtime,
    })))
  }
}

impl<'a> runtime::Caller<'a> for DynamicCaller<'a> {
  fn read_buf(&self, ptr: u32, buf: &mut [u8]) -> anyhow::Result<()> {
    match self.target {
      DynamicCallerTarget::Instance(instance) => self.runtime.ensure(
        unsafe { (self.runtime.instance_read_buf)(instance, ptr, buf.as_mut_ptr(), buf.len()) },
        "failed to read wasm instance memory",
      ),
      DynamicCallerTarget::ImportCaller(caller) => self.runtime.ensure(
        unsafe { (self.runtime.caller_read_buf)(caller, ptr, buf.as_mut_ptr(), buf.len()) },
        "failed to read wasm caller memory",
      ),
    }
  }

  fn write_buf(&mut self, ptr: u32, buf: &[u8]) -> anyhow::Result<()> {
    match self.target {
      DynamicCallerTarget::Instance(instance) => self.runtime.ensure(
        unsafe { (self.runtime.instance_write_buf)(instance, ptr, buf.as_ptr(), buf.len()) },
        "failed to write wasm instance memory",
      ),
      DynamicCallerTarget::ImportCaller(caller) => self.runtime.ensure(
        unsafe { (self.runtime.caller_write_buf)(caller, ptr, buf.as_ptr(), buf.len()) },
        "failed to write wasm caller memory",
      ),
    }
  }

  fn alloc(&mut self, size: u32) -> anyhow::Result<u32> {
    match self.target {
      DynamicCallerTarget::Instance(instance) => self.runtime.call_u32(
        |result| unsafe { (self.runtime.instance_alloc)(instance, size, result) },
        "failed to allocate wasm instance memory",
      ),
      DynamicCallerTarget::ImportCaller(caller) => self.runtime.call_u32(
        |result| unsafe { (self.runtime.caller_alloc)(caller, size, result) },
        "failed to allocate wasm caller memory",
      ),
    }
  }

  fn free(&mut self, ptr: u32, size: u32) -> anyhow::Result<u32> {
    match self.target {
      DynamicCallerTarget::Instance(instance) => self.runtime.call_u32(
        |result| unsafe { (self.runtime.instance_free)(instance, ptr, size, result) },
        "failed to free wasm instance memory",
      ),
      DynamicCallerTarget::ImportCaller(caller) => self.runtime.call_u32(
        |result| unsafe { (self.runtime.caller_free)(caller, ptr, size, result) },
        "failed to free wasm caller memory",
      ),
    }
  }
}

unsafe extern "C" fn call_import(
  data: *mut c_void,
  caller: *mut c_void,
  input: *const i32,
  input_len: usize,
  output: *mut i32,
  output_len: usize,
) {
  let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
    if data.is_null() || caller.is_null() {
      return;
    }

    let data = unsafe { &*(data.cast::<ImportData>()) };
    let input = if input.is_null() {
      &[]
    } else {
      unsafe { slice::from_raw_parts(input, input_len) }
    };
    let output = if output.is_null() {
      &mut []
    } else {
      unsafe { slice::from_raw_parts_mut(output, output_len) }
    };
    let mut caller = DynamicCaller {
      runtime: data.runtime,
      target: DynamicCallerTarget::ImportCaller(caller),
      _marker: PhantomData,
    };
    (data.func.func)(&mut caller, input, output);
  }));
}
