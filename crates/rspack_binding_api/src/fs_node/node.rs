use napi::{
  Either,
  bindgen_prelude::{Buffer, Either3, FnArgs, Promise},
};
use napi_derive::napi;
use rspack_fs::FileMetadata;
use rspack_napi::threadsafe_function::ThreadsafeFunction;

type Open = ThreadsafeFunction<FnArgs<(String, String)>, Promise<Either<i32, ()>>>;
type Write = ThreadsafeFunction<FnArgs<(i32, Buffer, u32)>, Promise<Either<u32, ()>>>;
type Read = ThreadsafeFunction<FnArgs<(i32, u32, u32)>, Promise<Either<Buffer, ()>>>;
type ReadUtil = ThreadsafeFunction<FnArgs<(i32, u8, u32)>, Promise<Either<Buffer, ()>>>;
type ReadToEnd = ThreadsafeFunction<FnArgs<(i32, u32)>, Promise<Either<Buffer, ()>>>;
type Chmod = ThreadsafeFunction<FnArgs<(String, u32)>, Promise<()>>;

#[derive(Debug)]
#[napi(object, object_to_js = false, js_name = "ThreadsafeNodeFS")]
pub struct ThreadsafeNodeFS {
  #[napi(ts_type = "(name: string, content: Buffer) => Promise<void>")]
  pub write_file: ThreadsafeFunction<FnArgs<(String, Buffer)>, Promise<()>>,
  #[napi(ts_type = "(name: string) => Promise<void>")]
  pub remove_file: ThreadsafeFunction<String, Promise<()>>,
  #[napi(ts_type = "(name: string) => Promise<void>")]
  pub mkdir: ThreadsafeFunction<String, Promise<()>>,
  #[napi(ts_type = "(name: string) => Promise<string | void>")]
  pub mkdirp: ThreadsafeFunction<String, Promise<Either<String, ()>>>,
  #[napi(ts_type = "(name: string) => Promise<string | void>")]
  pub remove_dir_all: ThreadsafeFunction<String, Promise<Either<String, ()>>>,
  #[napi(ts_type = "(name: string) => Promise<string[] | void>")]
  pub read_dir: ThreadsafeFunction<String, Promise<Either<Vec<String>, ()>>>,
  #[napi(ts_type = "(name: string) => Promise<Buffer | string | void>")]
  pub read_file: ThreadsafeFunction<String, Promise<Either3<Buffer, String, ()>>>,
  #[napi(ts_type = "(name: string) => Promise<NodeFsStats | void>")]
  pub stat: ThreadsafeFunction<String, Promise<Either<NodeFsStats, ()>>>,
  #[napi(ts_type = "(name: string) => Promise<NodeFsStats | void>")]
  pub lstat: ThreadsafeFunction<String, Promise<Either<NodeFsStats, ()>>>,
  #[napi(ts_type = "(name: string) => Promise<string | void>")]
  pub realpath: ThreadsafeFunction<String, Promise<Either<String, ()>>>,
  #[napi(ts_type = "(name: string, flags: string) => Promise<number | void>")]
  pub open: Open,
  #[napi(ts_type = "(from: string, to: string) => Promise<void>")]
  pub rename: ThreadsafeFunction<FnArgs<(String, String)>, Promise<()>>,
  #[napi(ts_type = "(fd: number) => Promise<void>")]
  pub close: ThreadsafeFunction<i32, Promise<()>>,
  #[napi(ts_type = "(fd: number, content: Buffer, position: number) => Promise<number | void>")]
  pub write: Write,
  #[napi(ts_type = "(fd: number, content: Buffer) => Promise<number | void>")]
  pub write_all: ThreadsafeFunction<FnArgs<(i32, Buffer)>, Promise<()>>,
  #[napi(ts_type = "(fd: number, length: number, position: number) => Promise<Buffer | void>")]
  pub read: Read,
  #[napi(ts_type = "(fd: number, code: number, position: number) => Promise<Buffer | void>")]
  pub read_until: ReadUtil,
  #[napi(ts_type = "(fd: number, position: number) => Promise<Buffer | void>")]
  pub read_to_end: ReadToEnd,
  // The following functions are not supported by webpack, so they are optional
  #[napi(ts_type = "(name: string, mode: number) => Promise<void>")]
  pub chmod: Option<Chmod>,
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
