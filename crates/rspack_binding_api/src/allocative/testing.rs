//! Native fixtures consumed by the allocative-object-markers config case.
//! Compiled only with both allocative and test-loader; absent from release APIs.
use std::sync::Arc;

#[napi]
pub struct AllocativeTestOwner {
  bytes: Arc<Vec<u8>>,
}

#[napi]
impl AllocativeTestOwner {
  #[napi(constructor, skip_typescript)]
  pub fn new(size: u32) -> Self {
    Self {
      bytes: Arc::new(vec![0; size as usize]),
    }
  }

  #[napi(factory, skip_typescript)]
  pub fn from_owner(owner: &AllocativeTestOwner) -> Self {
    Self {
      bytes: Arc::clone(&owner.bytes),
    }
  }

  #[napi(skip_typescript)]
  pub fn alias(&self) -> Self {
    Self {
      bytes: Arc::clone(&self.bytes),
    }
  }
}

use rspack_napi::napi::{
  Env,
  bindgen_prelude::{Reference, Result, SharedReference},
};

struct TestPayload {
  bytes: Arc<Vec<u8>>,
}

#[napi]
pub struct AllocativeTestPayload {
  inner: SharedReference<AllocativeTestOwner, TestPayload>,
}

#[napi]
impl AllocativeTestOwner {
  #[napi(skip_typescript)]
  pub fn payload(
    &self,
    reference: Reference<AllocativeTestOwner>,
    env: Env,
    size: u32,
  ) -> Result<AllocativeTestPayload> {
    Ok(AllocativeTestPayload {
      inner: reference.share_with(env, |_| {
        Ok(TestPayload {
          bytes: Arc::new(vec![0; size as usize]),
        })
      })?,
    })
  }
}

#[napi]
impl AllocativeTestPayload {
  #[napi(skip_typescript)]
  pub fn alias(&self, env: Env) -> Result<Self> {
    Ok(Self {
      inner: self.inner.clone(env)?,
    })
  }
}

#[napi]
impl AllocativeTestPayload {
  #[napi(skip_typescript)]
  pub fn chained(&self, env: Env) -> Result<Self> {
    Ok(Self {
      inner: self.inner.clone(env)?.share_with(env, |payload| {
        Ok(TestPayload {
          bytes: Arc::clone(&payload.bytes),
        })
      })?,
    })
  }

  #[napi(skip_typescript)]
  pub fn from_weak_owner(&self, env: Env) -> Result<Self> {
    let weak = self.inner.clone_owner(env)?.downgrade();
    let owner = weak.upgrade(env)?.expect("payload keeps owner alive");
    let bytes = Arc::clone(&self.inner.bytes);
    Ok(Self {
      inner: owner.share_with(env, |_| Ok(TestPayload { bytes }))?,
    })
  }
}

// Both public import paths must select the same tracking types.
#[napi(skip_typescript)]
pub fn allocative_test_uses_tracked_prelude() -> bool {
  use std::any::TypeId;

  use napi::bindgen_prelude as local;
  use rspack_napi::napi::bindgen_prelude as exported;

  TypeId::of::<local::Reference<AllocativeTestOwner>>()
    == TypeId::of::<exported::Reference<AllocativeTestOwner>>()
    && TypeId::of::<local::SharedReference<AllocativeTestOwner, TestPayload>>()
      == TypeId::of::<exported::SharedReference<AllocativeTestOwner, TestPayload>>()
    && TypeId::of::<local::WeakReference<AllocativeTestOwner>>()
      == TypeId::of::<exported::WeakReference<AllocativeTestOwner>>()
    && TypeId::of::<local::External<TestPayload>>()
      == TypeId::of::<exported::External<TestPayload>>()
}
