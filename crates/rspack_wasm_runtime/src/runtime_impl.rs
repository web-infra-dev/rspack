//! temp fork from https://github.com/swc-project/swc/blob/main/crates/swc_plugin_backend_wasmtime/src/lib.rs

use std::{
  cell::RefCell,
  ffi::{CString, c_char, c_void},
  path::PathBuf,
  ptr, slice, str,
};

use anyhow::{Context, bail};
use once_cell::sync::OnceCell;
static ENGINE: OnceCell<wasmtime::Engine> = OnceCell::new();

thread_local! {
  static LAST_ERROR: RefCell<Option<CString>> = const { RefCell::new(None) };
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct BytesView {
  pub ptr: *const u8,
  pub len: usize,
}

pub type ImportCallback = unsafe extern "C" fn(
  data: *mut c_void,
  caller: *mut c_void,
  input: *const i32,
  input_len: usize,
  output: *mut i32,
  output_len: usize,
);

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Import {
  pub name: BytesView,
  pub params: u8,
  pub results: u8,
  pub data: *mut c_void,
  pub callback: ImportCallback,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Env {
  pub key: BytesView,
  pub value: BytesView,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Module {
  pub kind: u8,
  pub bytes: BytesView,
  pub cache: *mut c_void,
}

fn set_last_error(error: impl std::fmt::Display) -> bool {
  let message = error.to_string().replace('\0', "\\0");
  LAST_ERROR.with(|last_error| {
    *last_error.borrow_mut() =
      Some(CString::new(message).expect("NUL bytes are replaced before constructing CString"));
  });
  false
}

pub fn last_error() -> *const c_char {
  LAST_ERROR.with(|last_error| {
    last_error
      .borrow()
      .as_ref()
      .map_or(ptr::null(), |error| error.as_ptr())
  })
}

unsafe fn bytes<'a>(value: BytesView) -> &'a [u8] {
  if value.ptr.is_null() {
    &[]
  } else {
    unsafe { slice::from_raw_parts(value.ptr, value.len) }
  }
}

unsafe fn string(value: BytesView) -> anyhow::Result<String> {
  Ok(str::from_utf8(unsafe { bytes(value) })?.to_string())
}

unsafe fn path(value: BytesView) -> anyhow::Result<PathBuf> {
  Ok(PathBuf::from(unsafe { string(value) }?))
}

unsafe fn ffi_slice<'a, T>(ptr: *const T, len: usize) -> anyhow::Result<&'a [T]> {
  if ptr.is_null() {
    if len == 0 {
      Ok(&[])
    } else {
      bail!("slice pointer is null but length is {len}");
    }
  } else {
    Ok(unsafe { slice::from_raw_parts(ptr, len) })
  }
}

struct WasmtimeCache(wasmtime::Module);

struct WasmtimeTable {
  memory: Option<wasmtime::Memory>,
  alloc_func: Option<wasmtime::TypedFunc<u32, u32>>,
  free_func: Option<wasmtime::TypedFunc<(u32, u32), u32>>,

  wasi: wasi_common::WasiCtx,
}

struct WasmtimeInstance {
  instance: wasmtime::Instance,
  store: wasmtime::Store<WasmtimeTable>,

  memory: wasmtime::Memory,
  alloc_func: wasmtime::TypedFunc<u32, u32>,
  free_func: wasmtime::TypedFunc<(u32, u32), u32>,
  transform_func: wasmtime::TypedFunc<(u32, u32, u32, u32), u32>,
}

struct WasmtimeCallerRef<'a> {
  caller: wasmtime::Caller<'a, WasmtimeTable>,
  memory: wasmtime::Memory,
  alloc_func: wasmtime::TypedFunc<u32, u32>,
  free_func: wasmtime::TypedFunc<(u32, u32), u32>,
}

fn init_engine() -> anyhow::Result<wasmtime::Engine> {
  let config = wasmtime::Config::default();
  wasmtime::Engine::new(&config)
}

pub unsafe fn prepare_module(bytes_view: BytesView, cache: *mut *mut c_void) -> bool {
  if cache.is_null() {
    return set_last_error("cache output pointer is null");
  }

  let result = (|| -> anyhow::Result<*mut c_void> {
    let engine = ENGINE.get_or_try_init(init_engine)?;
    let cache = WasmtimeCache(wasmtime::Module::new(engine, unsafe { bytes(bytes_view) })?);
    Ok(Box::into_raw(Box::new(cache)).cast())
  })();

  match result {
    Ok(value) => {
      unsafe { *cache = value };
      true
    }
    Err(error) => set_last_error(error),
  }
}

pub unsafe fn clone_cache(cache: *mut c_void, cloned: *mut *mut c_void) -> bool {
  if cache.is_null() || cloned.is_null() {
    return set_last_error("cache pointer is null");
  }

  let WasmtimeCache(module) = unsafe { &*cache.cast::<WasmtimeCache>() };
  let cache = WasmtimeCache(module.clone());
  unsafe { *cloned = Box::into_raw(Box::new(cache)).cast() };
  true
}

pub unsafe fn load_cache(path_view: BytesView, cache: *mut *mut c_void) -> bool {
  if cache.is_null() {
    return set_last_error("cache output pointer is null");
  }

  let Some(value) = (|| -> Option<*mut c_void> {
    let module = std::fs::read(unsafe { path(path_view).ok()? }).ok()?;
    let engine = ENGINE.get_or_try_init(init_engine).ok()?;
    let cache = unsafe { wasmtime::Module::deserialize(engine, module).ok()? };
    Some(Box::into_raw(Box::new(WasmtimeCache(cache))).cast())
  })() else {
    unsafe { *cache = ptr::null_mut() };
    return true;
  };

  unsafe { *cache = value };
  true
}

pub unsafe fn store_cache(path_view: BytesView, cache: *mut c_void) -> bool {
  if cache.is_null() {
    return set_last_error("cache pointer is null");
  }

  let result = (|| -> anyhow::Result<()> {
    use std::io::{ErrorKind, Write};

    let path = unsafe { path(path_view) }?;
    let WasmtimeCache(module) = unsafe { &*cache.cast::<WasmtimeCache>() };
    let data = module.serialize()?;
    let tmppath = {
      let mut ext = path.extension().unwrap_or_default().to_owned();
      ext.push(".tmp");
      path.with_extension(ext)
    };
    let mut fd = match std::fs::OpenOptions::new()
      .create_new(true)
      .write(true)
      .open(&tmppath)
    {
      Ok(fd) => fd,
      Err(ref err) if err.kind() == ErrorKind::AlreadyExists => return Ok(()),
      Err(err) => return Err(err.into()),
    };
    fd.write_all(&data)?;
    drop(fd);
    std::fs::rename(&tmppath, path)?;
    Ok(())
  })();

  match result {
    Ok(()) => true,
    Err(error) => set_last_error(error),
  }
}

pub unsafe fn destroy_cache(cache: *mut c_void) {
  if !cache.is_null() {
    drop(unsafe { Box::from_raw(cache.cast::<WasmtimeCache>()) });
  }
}

pub unsafe fn init(
  _name: BytesView,
  imports: *const Import,
  imports_len: usize,
  envs: *const Env,
  envs_len: usize,
  module: Module,
  instance: *mut *mut c_void,
) -> bool {
  if instance.is_null() {
    return set_last_error("instance output pointer is null");
  }

  let result = (|| -> anyhow::Result<*mut c_void> {
    let engine = ENGINE.get_or_try_init(init_engine)?;

    let module = match module.kind {
      0 => wasmtime::Module::new(engine, unsafe { bytes(module.bytes) })?,
      1 => {
        if module.cache.is_null() {
          bail!("module cache pointer is null");
        }
        let cache = unsafe { Box::from_raw(module.cache.cast::<WasmtimeCache>()) };
        cache.0
      }
      kind => bail!("unsupported module kind {kind}"),
    };

    let envs = unsafe { ffi_slice(envs, envs_len) }?
      .iter()
      .map(|env| Ok((unsafe { string(env.key) }?, unsafe { string(env.value) }?)))
      .collect::<anyhow::Result<Vec<_>>>()?;

    let current_dir = std::env::current_dir()?;
    let dir = wasi_common::sync::Dir::open_ambient_dir(
      &current_dir,
      wasi_common::sync::ambient_authority(),
    )?;
    let wasi = wasi_common::sync::WasiCtxBuilder::new()
      .envs(&envs)?
      .preopened_dir(dir, "/cwd")?
      .build();

    let table = WasmtimeTable {
      memory: None,
      alloc_func: None,
      free_func: None,

      wasi,
    };
    let mut linker: wasmtime::Linker<WasmtimeTable> = wasmtime::Linker::new(engine);
    for import in unsafe { ffi_slice(imports, imports_len) }? {
      let name = unsafe { string(import.name) }?;
      let ty = wasmtime::FuncType::new(
        engine,
        (0..import.params).map(|_| wasmtime::ValType::I32),
        (0..import.results).map(|_| wasmtime::ValType::I32),
      );
      let data = import.data as usize;
      let callback = import.callback;
      linker.func_new("env", &name, ty, move |caller, input, output| {
        ffi_wasmtime_func_call(caller, input, output, data as *mut c_void, callback)
      })?;
    }

    wasi_common::sync::add_to_linker(&mut linker, |t| &mut t.wasi)?;

    let mut store = wasmtime::Store::new(engine, table);
    let instance = linker.instantiate(&mut store, &module)?;

    let memory = instance
      .get_memory(&mut store, "memory")
      .context("miss memory export")?;
    let alloc_func: wasmtime::TypedFunc<u32, u32> =
      instance.get_typed_func(&mut store, "__alloc")?;
    let free_func: wasmtime::TypedFunc<(u32, u32), u32> =
      instance.get_typed_func(&mut store, "__free")?;
    let transform_func: wasmtime::TypedFunc<(u32, u32, u32, u32), u32> =
      instance.get_typed_func(&mut store, "__transform_plugin_process_impl")?;

    store.data_mut().memory = Some(memory);
    store.data_mut().alloc_func = Some(alloc_func.clone());
    store.data_mut().free_func = Some(free_func.clone());

    instance
      .get_typed_func::<(), u32>(&mut store, "__get_transform_plugin_core_pkg_diag")?
      .call(&mut store, ())?;

    Ok(
      Box::into_raw(Box::new(WasmtimeInstance {
        store,
        instance,
        memory,
        alloc_func,
        free_func,
        transform_func,
      }))
      .cast(),
    )
  })();

  match result {
    Ok(value) => {
      unsafe { *instance = value };
      true
    }
    Err(error) => set_last_error(error),
  }
}

fn ffi_wasmtime_func_call(
  caller: wasmtime::Caller<'_, WasmtimeTable>,
  input: &[wasmtime::Val],
  output: &mut [wasmtime::Val],
  data: *mut c_void,
  callback: ImportCallback,
) -> anyhow::Result<()> {
  let memory = caller.data().memory.unwrap();
  let alloc_func = caller.data().alloc_func.clone().unwrap();
  let free_func = caller.data().free_func.clone().unwrap();
  let mut caller = WasmtimeCallerRef {
    caller,
    memory,
    alloc_func,
    free_func,
  };
  let input = input
    .iter()
    .map(|val| match val {
      wasmtime::Val::I32(v) => Ok(*v),
      _ => Err(anyhow::format_err!("not support argument type")),
    })
    .collect::<anyhow::Result<Vec<i32>>>()?;
  let mut output2 = vec![0; output.len()];

  unsafe {
    callback(
      data,
      (&mut caller as *mut WasmtimeCallerRef<'_>).cast(),
      input.as_ptr(),
      input.len(),
      output2.as_mut_ptr(),
      output2.len(),
    );
  }

  for i in 0..output.len() {
    output[i] = wasmtime::Val::I32(output2[i]);
  }

  Ok(())
}

pub unsafe fn instance_transform(
  instance: *mut c_void,
  program_ptr: u32,
  program_len: u32,
  unresolved_mark: u32,
  should_enable_comments_proxy: u32,
  result: *mut u32,
) -> bool {
  if instance.is_null() || result.is_null() {
    return set_last_error("instance or result pointer is null");
  }

  let instance = unsafe { &mut *instance.cast::<WasmtimeInstance>() };
  match instance.transform_func.call(
    &mut instance.store,
    (
      program_ptr,
      program_len,
      unresolved_mark,
      should_enable_comments_proxy,
    ),
  ) {
    Ok(value) => {
      unsafe { *result = value };
      true
    }
    Err(error) => set_last_error(error),
  }
}

pub unsafe fn instance_read_buf(instance: *mut c_void, ptr: u32, buf: *mut u8, len: usize) -> bool {
  if instance.is_null() || buf.is_null() {
    return set_last_error("instance or buffer pointer is null");
  }

  let instance = unsafe { &mut *instance.cast::<WasmtimeInstance>() };
  match instance.memory.read(&instance.store, ptr as usize, unsafe {
    slice::from_raw_parts_mut(buf, len)
  }) {
    Ok(()) => true,
    Err(error) => set_last_error(error),
  }
}

pub unsafe fn instance_write_buf(
  instance: *mut c_void,
  ptr: u32,
  buf: *const u8,
  len: usize,
) -> bool {
  if instance.is_null() || buf.is_null() {
    return set_last_error("instance or buffer pointer is null");
  }

  let instance = unsafe { &mut *instance.cast::<WasmtimeInstance>() };
  match instance
    .memory
    .write(&mut instance.store, ptr as usize, unsafe {
      slice::from_raw_parts(buf, len)
    }) {
    Ok(()) => true,
    Err(error) => set_last_error(error),
  }
}

pub unsafe fn instance_alloc(instance: *mut c_void, size: u32, result: *mut u32) -> bool {
  if instance.is_null() || result.is_null() {
    return set_last_error("instance or result pointer is null");
  }

  let instance = unsafe { &mut *instance.cast::<WasmtimeInstance>() };
  match instance.alloc_func.call(&mut instance.store, size) {
    Ok(value) => {
      unsafe { *result = value };
      true
    }
    Err(error) => set_last_error(error),
  }
}

pub unsafe fn instance_free(instance: *mut c_void, ptr: u32, size: u32, result: *mut u32) -> bool {
  if instance.is_null() || result.is_null() {
    return set_last_error("instance or result pointer is null");
  }

  let instance = unsafe { &mut *instance.cast::<WasmtimeInstance>() };
  match instance.free_func.call(&mut instance.store, (ptr, size)) {
    Ok(value) => {
      unsafe { *result = value };
      true
    }
    Err(error) => set_last_error(error),
  }
}

pub unsafe fn instance_cache(instance: *mut c_void, cache: *mut *mut c_void) -> bool {
  if instance.is_null() || cache.is_null() {
    return set_last_error("instance or cache output pointer is null");
  }

  let instance = unsafe { &mut *instance.cast::<WasmtimeInstance>() };
  let module = instance.instance.module(&instance.store);
  let cache = unsafe { &mut *cache };
  *cache = Box::into_raw(Box::new(WasmtimeCache(module.clone()))).cast();
  true
}

pub unsafe fn destroy_instance(instance: *mut c_void) {
  if !instance.is_null() {
    drop(unsafe { Box::from_raw(instance.cast::<WasmtimeInstance>()) });
  }
}

pub unsafe fn caller_read_buf(caller: *mut c_void, ptr: u32, buf: *mut u8, len: usize) -> bool {
  if caller.is_null() || buf.is_null() {
    return set_last_error("caller or buffer pointer is null");
  }

  let caller = unsafe { &mut *caller.cast::<WasmtimeCallerRef<'_>>() };
  match caller.memory.read(&caller.caller, ptr as usize, unsafe {
    slice::from_raw_parts_mut(buf, len)
  }) {
    Ok(()) => true,
    Err(error) => set_last_error(error),
  }
}

pub unsafe fn caller_write_buf(caller: *mut c_void, ptr: u32, buf: *const u8, len: usize) -> bool {
  if caller.is_null() || buf.is_null() {
    return set_last_error("caller or buffer pointer is null");
  }

  let caller = unsafe { &mut *caller.cast::<WasmtimeCallerRef<'_>>() };
  match caller
    .memory
    .write(&mut caller.caller, ptr as usize, unsafe {
      slice::from_raw_parts(buf, len)
    }) {
    Ok(()) => true,
    Err(error) => set_last_error(error),
  }
}

pub unsafe fn caller_alloc(caller: *mut c_void, size: u32, result: *mut u32) -> bool {
  if caller.is_null() || result.is_null() {
    return set_last_error("caller or result pointer is null");
  }

  let caller = unsafe { &mut *caller.cast::<WasmtimeCallerRef<'_>>() };
  match caller.alloc_func.call(&mut caller.caller, size) {
    Ok(value) => {
      unsafe { *result = value };
      true
    }
    Err(error) => set_last_error(error),
  }
}

pub unsafe fn caller_free(caller: *mut c_void, ptr: u32, size: u32, result: *mut u32) -> bool {
  if caller.is_null() || result.is_null() {
    return set_last_error("caller or result pointer is null");
  }

  let caller = unsafe { &mut *caller.cast::<WasmtimeCallerRef<'_>>() };
  match caller.free_func.call(&mut caller.caller, (ptr, size)) {
    Ok(value) => {
      unsafe { *result = value };
      true
    }
    Err(error) => set_last_error(error),
  }
}
