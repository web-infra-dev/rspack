use std::{
  path::Path,
  sync::{Arc, Mutex},
};

use rspack_cacheable::{
  __private::rkyv::{Archive, Deserialize, Serialize, bytecheck::CheckBytes},
  Deserializer, Serializer, Validator, from_bytes, to_bytes,
};
use rspack_error::Result;
use rspack_paths::Utf8PathBuf;
use rustc_hash::FxHashMap;

/// Internal cacheable context for serialization
#[derive(Debug, Clone)]
struct Context {
  portable_project_root: Option<Utf8PathBuf>,
  imported_dependency_ids: Option<Arc<Mutex<FxHashMap<u32, u32>>>>,
}

impl rspack_cacheable::CacheableContext for Context {
  fn project_root(&self) -> Option<&Path> {
    self.portable_project_root.as_ref().map(|p| p.as_std_path())
  }
}

/// Cache codec for encoding and decoding cacheable data
///
/// This struct encapsulates the serialization and deserialization logic,
/// automatically passing the project context to rspack_cacheable's to_bytes and from_bytes.
///
/// # Example
///
/// ```ignore
/// let codec = CacheCodec::new(portable_project_root);
///
/// // Encode data to bytes
/// let bytes = codec.encode(&my_data)?;
///
/// // Decode bytes back to data
/// let my_data: MyType = codec.decode(&bytes)?;
/// ```
#[derive(Debug, Clone)]
pub struct CacheCodec {
  context: Context,
}

impl CacheCodec {
  pub fn new(portable_project_root: Option<Utf8PathBuf>) -> Self {
    Self {
      context: Context {
        portable_project_root,
        imported_dependency_ids: None,
      },
    }
  }

  pub(crate) fn for_import() -> Self {
    Self {
      context: Context {
        portable_project_root: None,
        imported_dependency_ids: Some(Default::default()),
      },
    }
  }

  pub(crate) fn restore_dependency_id(id: u32, guard: &rspack_cacheable::ContextGuard) -> u32 {
    let Ok(context) = guard.downcast_context::<Context>() else {
      return id;
    };
    let Some(ids) = &context.imported_dependency_ids else {
      return id;
    };
    *ids
      .lock()
      .expect("imported dependency ID lock poisoned")
      .entry(id)
      .or_insert_with(rspack_tasks::fetch_new_dependency_id)
  }

  pub fn encode<T>(&self, data: &T) -> Result<Vec<u8>>
  where
    T: for<'a> Serialize<Serializer<'a>>,
  {
    to_bytes(data, &self.context).map_err(rspack_error::Error::from_error)
  }

  pub fn decode<T>(&self, bytes: &[u8]) -> Result<T>
  where
    T: Archive,
    T::Archived: for<'a> CheckBytes<Validator<'a>> + Deserialize<T, Deserializer>,
  {
    from_bytes(bytes, &self.context).map_err(rspack_error::Error::from_error)
  }
}
