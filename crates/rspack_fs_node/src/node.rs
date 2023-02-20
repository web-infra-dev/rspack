use napi::{Env, JsFunction, Ref};
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

#[napi(object)]
pub struct NodeFS {
  pub write_file: JsFunction,
  pub mkdir: JsFunction,
  pub mkdirp: JsFunction,
}

pub(crate) trait TryIntoNodeFSRef {
  fn try_into_node_fs_ref(self, env: &Env) -> napi::Result<NodeFSRef>;
}

impl TryIntoNodeFSRef for NodeFS {
  fn try_into_node_fs_ref(self, env: &Env) -> napi::Result<NodeFSRef> {
    Ok(NodeFSRef {
      write_file: JsFunctionRef::new(env.clone(), self.write_file)?,
      mkdir: JsFunctionRef::new(env.clone(), self.mkdir)?,
      mkdirp: JsFunctionRef::new(env.clone(), self.mkdirp)?,
    })
  }
}

pub(crate) struct NodeFSRef {
  pub(crate) write_file: JsFunctionRef,
  pub(crate) mkdir: JsFunctionRef,
  pub(crate) mkdirp: JsFunctionRef,
}
