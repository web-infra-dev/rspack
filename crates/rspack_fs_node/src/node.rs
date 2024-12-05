use napi::bindgen_prelude::{Buffer, Either3};
use napi::Either;
use napi_derive::napi;
use rspack_fs::FileMetadata;
use rspack_napi::threadsafe_function::ThreadsafeFunction;

#[derive(Debug)]
#[napi(object, object_to_js = false, js_name = "ThreadsafeNodeFS")]
pub struct ThreadsafeNodeFS {
  #[napi(ts_type = "(name: string, content: Buffer) => Promise<void> | void")]
  pub write_file: ThreadsafeFunction<(String, Buffer), ()>,
  #[napi(ts_type = "(name: string) => Promise<void> | void")]
  pub remove_file: ThreadsafeFunction<String, ()>,
  #[napi(ts_type = "(name: string) => Promise<void> | void")]
  pub mkdir: ThreadsafeFunction<String, ()>,
  #[napi(ts_type = "(name: string) => Promise<string | void> | string | void")]
  pub mkdirp: ThreadsafeFunction<String, Either<String, ()>>,
  #[napi(ts_type = "(name: string) => Promise<string | void> | string | void")]
  pub remove_dir_all: ThreadsafeFunction<String, Either<String, ()>>,
  #[napi(ts_type = "(name: string) => Promise<string[] | void> | string[] | void")]
  pub read_dir: ThreadsafeFunction<String, Either<Vec<String>, ()>>,
  #[napi(ts_type = "(name: string) => Promise<Buffer | string | void> | Buffer | string | void")]
  pub read_file: ThreadsafeFunction<String, Either3<Buffer, String, ()>>,
  #[napi(ts_type = "(name: string) => Promise<NodeFsStats | void> | NodeFsStats | void")]
  pub stat: ThreadsafeFunction<String, Either<NodeFsStats, ()>>,
  #[napi(ts_type = "(name: string) => Promise<NodeFsStats | void> | NodeFsStats | void")]
  pub lstat: ThreadsafeFunction<String, Either<NodeFsStats, ()>>,
  #[napi(ts_type = "(name: string, flags: string) => Promise<number | void> | number | void")]
  pub open: ThreadsafeFunction<(String, String), Either<i32, ()>>,
  #[napi(ts_type = "(from: string, to: string) => Promise<void> | void")]
  pub rename: ThreadsafeFunction<(String, String), ()>,
  #[napi(ts_type = "(fd: number) => Promise<void> | void")]
  pub close: ThreadsafeFunction<i32, ()>,
  #[napi(
    ts_type = "(fd: number, content: Buffer, position: number) => Promise<number | void> | number | void"
  )]
  pub write: ThreadsafeFunction<(i32, Buffer, u32), Either<u32, ()>>,
  #[napi(ts_type = "(fd: number, content: Buffer) => Promise<number | void> | number | void")]
  pub write_all: ThreadsafeFunction<(i32, Buffer), ()>,
  #[napi(
    ts_type = "(fd: number, length: number, position: number) => Promise<Buffer | void> | Buffer | void"
  )]
  pub read: ThreadsafeFunction<(i32, u32, u32), Either<Buffer, ()>>,
  #[napi(
    ts_type = "(fd: number, code: number, position: number) => Promise<Buffer | void> | Buffer | void"
  )]
  pub read_until: ThreadsafeFunction<(i32, u8, u32), Either<Buffer, ()>>,
  #[napi(ts_type = "(fd: number, position: number) => Promise<Buffer | void> | Buffer | void")]
  pub read_to_end: ThreadsafeFunction<(i32, u32), Either<Buffer, ()>>,
}

#[napi(object, object_to_js = false)]
pub struct NodeFsStats {
  pub is_file: bool,
  pub is_directory: bool,
  pub atime_ms: u32,
  pub mtime_ms: u32,
  pub ctime_ms: u32,
  pub birthtime_ms: u32,
  pub size: u32,
}

impl From<NodeFsStats> for FileMetadata {
  fn from(value: NodeFsStats) -> Self {
    Self {
      is_file: value.is_file,
      is_directory: value.is_directory,
      is_symlink: false,
      atime_ms: value.atime_ms as u64,
      mtime_ms: value.mtime_ms as u64,
      ctime_ms: value.ctime_ms as u64,
      size: value.size as u64,
    }
  }
}
