use napi_derive::napi;
use rspack_fs::FileMetadata;
use rspack_napi::threadsafe_function::DynThreadsafeFunction;

#[derive(Debug)]
#[napi(object, object_to_js = false, js_name = "ThreadsafeNodeFS")]
pub struct ThreadsafeNodeFS {
  #[napi(ts_type = "(name: string, content: Buffer) => Promise<void>")]
  pub write_file: DynThreadsafeFunction,
  #[napi(ts_type = "(name: string) => Promise<void>")]
  pub remove_file: DynThreadsafeFunction,
  #[napi(ts_type = "(name: string) => Promise<void>")]
  pub mkdir: DynThreadsafeFunction,
  #[napi(ts_type = "(name: string) => Promise<string | void>")]
  pub mkdirp: DynThreadsafeFunction,
  #[napi(ts_type = "(name: string) => Promise<string | void>")]
  pub remove_dir_all: DynThreadsafeFunction,
  #[napi(ts_type = "(name: string) => Promise<string[] | void>")]
  pub read_dir: DynThreadsafeFunction,
  #[napi(ts_type = "(name: string) => Promise<Buffer | string | void>")]
  pub read_file: DynThreadsafeFunction,
  #[napi(ts_type = "(name: string) => Promise<NodeFsStats | void>")]
  pub stat: DynThreadsafeFunction,
  #[napi(ts_type = "(name: string) => Promise<NodeFsStats | void>")]
  pub lstat: DynThreadsafeFunction,
  #[napi(ts_type = "(name: string) => Promise<string | void>")]
  pub realpath: DynThreadsafeFunction,
  #[napi(ts_type = "(name: string, flags: string) => Promise<number | void>")]
  pub open: DynThreadsafeFunction,
  #[napi(ts_type = "(from: string, to: string) => Promise<void>")]
  pub rename: DynThreadsafeFunction,
  #[napi(ts_type = "(fd: number) => Promise<void>")]
  pub close: DynThreadsafeFunction,
  #[napi(ts_type = "(fd: number, content: Buffer, position: number) => Promise<number | void>")]
  pub write: DynThreadsafeFunction,
  #[napi(ts_type = "(fd: number, content: Buffer) => Promise<number | void>")]
  pub write_all: DynThreadsafeFunction,
  #[napi(ts_type = "(fd: number, length: number, position: number) => Promise<Buffer | void>")]
  pub read: DynThreadsafeFunction,
  #[napi(ts_type = "(fd: number, code: number, position: number) => Promise<Buffer | void>")]
  pub read_until: DynThreadsafeFunction,
  #[napi(ts_type = "(fd: number, position: number) => Promise<Buffer | void>")]
  pub read_to_end: DynThreadsafeFunction,
  // The following functions are not supported by webpack, so they are optional
  #[napi(ts_type = "(name: string, mode: number) => Promise<void>")]
  pub chmod: Option<DynThreadsafeFunction>,
}

#[napi(object, object_to_js = false)]
pub struct NodeFsStats {
  pub is_file: bool,
  pub is_directory: bool,
  pub is_symlink: bool,
  pub atime_ms: u32,
  pub mtime_ms: u32,
  pub ctime_ms: u32,
  pub birthtime_ms: u32,
  pub size: u32,
  pub mode: u32,
}

impl From<NodeFsStats> for FileMetadata {
  fn from(value: NodeFsStats) -> Self {
    Self {
      is_file: value.is_file,
      is_directory: value.is_directory,
      is_symlink: value.is_symlink,
      atime_ms: value.atime_ms as u64,
      mtime_ms: value.mtime_ms as u64,
      ctime_ms: value.ctime_ms as u64,
      size: value.size as u64,
    }
  }
}
