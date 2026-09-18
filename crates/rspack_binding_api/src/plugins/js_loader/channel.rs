use std::sync::{Arc, LazyLock, RwLock, Weak};

use napi::{Env, bindgen_prelude::PromiseRaw};
use napi_derive::napi;
use rspack_core::CompilerId;
use rspack_error::{Result, error};
use rustc_hash::{FxHashMap, FxHashSet};
use tokio::sync::{Mutex, mpsc, oneshot};

use super::JsLoaderContext;

// Loaders can be restored from cache, so resolve their transport using the current
// compiler instead of serializing a sender. The registry does not keep compilers alive.
static DISPATCHERS: LazyLock<RwLock<FxHashMap<CompilerId, Weak<LoaderDispatcher>>>> =
  LazyLock::new(Default::default);

#[napi]
pub struct JsLoaderTask {
  #[napi(readonly)]
  pub compiler_id: u32,
  context: Option<JsLoaderContext>,
  reply: Option<oneshot::Sender<Result<JsLoaderContext>>>,
}

#[napi]
impl JsLoaderTask {
  #[napi]
  pub fn take_context(&mut self) -> napi::Result<JsLoaderContext> {
    self
      .context
      .take()
      .ok_or_else(|| napi::Error::from_reason("Loader task context has already been taken"))
  }

  #[napi]
  pub fn reply(&mut self, context: JsLoaderContext) {
    self.context = None;
    if let Some(reply) = self.reply.take() {
      let _ = reply.send(Ok(context));
    }
  }

  #[napi]
  pub fn fail(&mut self, message: String) {
    self.context = None;
    if let Some(reply) = self.reply.take() {
      let _ = reply.send(Err(error!(message)));
    }
  }
}

/// Shared by the compilers using the JS loader task loop. A wake message lets the
/// loop stop receiving when all builds are idle, so a pending receive cannot keep Node alive.
#[napi]
pub struct JsLoaderChannel {
  sender: mpsc::UnboundedSender<Option<JsLoaderTask>>,
  receiver: Arc<Mutex<mpsc::UnboundedReceiver<Option<JsLoaderTask>>>>,
}

#[napi]
impl JsLoaderChannel {
  #[napi(constructor)]
  pub fn new() -> Self {
    let (sender, receiver) = mpsc::unbounded_channel();
    Self {
      sender,
      receiver: Arc::new(Mutex::new(receiver)),
    }
  }

  #[napi]
  pub fn receive<'env>(
    &self,
    env: &'env Env,
  ) -> napi::Result<PromiseRaw<'env, Option<JsLoaderTask>>> {
    let receiver = self.receiver.clone();
    rspack_napi::runtime::promise_from_future(env, async move {
      Ok(receiver.lock().await.recv().await.flatten())
    })
  }

  #[napi]
  pub fn wake(&self) {
    let _ = self.sender.send(None);
  }
}

pub(crate) struct LoaderDispatcher {
  compiler_id: CompilerId,
  sender: mpsc::UnboundedSender<Option<JsLoaderTask>>,
  pub loaders_without_pitch: RwLock<FxHashSet<String>>,
}

impl LoaderDispatcher {
  pub fn new(compiler_id: CompilerId, channel: &JsLoaderChannel) -> Arc<Self> {
    let dispatcher = Arc::new(Self {
      compiler_id,
      sender: channel.sender.clone(),
      loaders_without_pitch: Default::default(),
    });
    DISPATCHERS
      .write()
      .expect("should get lock")
      .insert(compiler_id, Arc::downgrade(&dispatcher));
    dispatcher
  }

  pub fn get(compiler_id: CompilerId) -> Result<Arc<Self>> {
    DISPATCHERS
      .read()
      .expect("should get lock")
      .get(&compiler_id)
      .and_then(Weak::upgrade)
      .ok_or_else(|| error!("The JavaScript loader dispatcher has been closed"))
  }

  pub async fn run(&self, context: JsLoaderContext) -> Result<JsLoaderContext> {
    let (reply, result) = oneshot::channel();
    self
      .sender
      .send(Some(JsLoaderTask {
        compiler_id: self.compiler_id.as_u32(),
        context: Some(context),
        reply: Some(reply),
      }))
      .map_err(|_| error!("The JavaScript loader task channel has been closed"))?;
    result
      .await
      .map_err(|_| error!("The JavaScript loader task was dropped without a reply"))?
  }

  pub fn close(&self) {
    DISPATCHERS
      .write()
      .expect("should get lock")
      .remove(&self.compiler_id);
  }
}

impl Drop for LoaderDispatcher {
  fn drop(&mut self) {
    self.close();
  }
}
