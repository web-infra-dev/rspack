use napi::{
  bindgen_prelude::{Buffer, Either3},
  Env, JsFunction, Ref,
};
use napi_derive::napi;

pub(crate) struct JsFunctionRef {
  env: Env,
  reference: Ref<()>,
}

impl JsFunctionRef {
  fn new(env: Env, f: JsFunction) -> napi::Result<Self> {
    Ok(Self {
      env,
      reference: env.create_reference(f)?,
    })
  }

  pub(crate) fn get(&self) -> napi::Result<JsFunction> {
    self.env.get_reference_value(&self.reference)
  }
}

impl Drop for JsFunctionRef {
  fn drop(&mut self) {
    let result = self.reference.unref(self.env);
    debug_assert!(result.is_ok());
  }
}

#[napi(object, js_name = "NodeFS")]
pub struct NodeFS {
  pub write_file: JsFunction,
  pub remove_file: JsFunction,
  pub mkdir: JsFunction,
  pub mkdirp: JsFunction,
}

pub(crate) trait TryIntoNodeFSRef {
  fn try_into_node_fs_ref(self, env: &Env) -> napi::Result<NodeFSRef>;
}

impl TryIntoNodeFSRef for NodeFS {
  fn try_into_node_fs_ref(self, env: &Env) -> napi::Result<NodeFSRef> {
    Ok(NodeFSRef {
      write_file: JsFunctionRef::new(*env, self.write_file)?,
      mkdir: JsFunctionRef::new(*env, self.mkdir)?,
      mkdirp: JsFunctionRef::new(*env, self.mkdirp)?,
    })
  }
}

pub(crate) struct NodeFSRef {
  pub(crate) write_file: JsFunctionRef,
  pub(crate) mkdir: JsFunctionRef,
  pub(crate) mkdirp: JsFunctionRef,
}

use napi::Either;
use rspack_fs::r#async::FileStat;
use rspack_napi::threadsafe_function::ThreadsafeFunction;

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
  #[napi(
    ts_type = "(name: string, options: NodeFsReadFileOptions) => Promise<Buffer | string | void> | Buffer | string | void"
  )]
  pub read_file: ThreadsafeFunction<String, Either3<Buffer, String, ()>>,
  #[napi(ts_type = "(name: string) => Promise<NodeFsStats | void> | NodeFsStats | void")]
  pub stat: ThreadsafeFunction<String, Either<NodeFsStats, ()>>,
  #[napi(ts_type = "(name: string) => Promise<NodeFsStats | void> | NodeFsStats | void")]
  pub lstat: ThreadsafeFunction<String, Either<NodeFsStats, ()>>,
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

impl From<NodeFsStats> for FileStat {
  fn from(value: NodeFsStats) -> Self {
    Self {
      is_file: value.is_file,
      is_directory: value.is_directory,
      atime_ms: value.atime_ms as u64,
      mtime_ms: value.mtime_ms as u64,
      ctime_ms: value.ctime_ms as u64,
      size: value.size as u64,
    }
  }
}
