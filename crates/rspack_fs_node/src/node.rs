use napi::{bindgen_prelude::Buffer, Env, JsFunction, Ref};
use napi_derive::napi;
use rspack_fs::cfg_async;

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

cfg_async! {
  use napi::Either;
  use rspack_napi::threadsafe_function::ThreadsafeFunction;

  #[napi(object, object_to_js = false, js_name = "ThreadsafeNodeFS")]
  pub struct ThreadsafeNodeFS {
    #[napi(ts_type = "(name: string, content: Buffer) => void")]
    pub write_file: ThreadsafeFunction<(String, Buffer), ()>,
    #[napi(ts_type = "(name: string) => void")]
    pub remove_file: ThreadsafeFunction<String, ()>,
    #[napi(ts_type = "(name: string) => void")]
    pub mkdir: ThreadsafeFunction<String, ()>,
    #[napi(ts_type = "(name: string) => string | void")]
    pub mkdirp: ThreadsafeFunction<String, Either<String, ()>>,
    #[napi(ts_type = "(name: string) => string | void")]
    pub remove_dir_all: ThreadsafeFunction<String, Either<String, ()>>,
  }
}
