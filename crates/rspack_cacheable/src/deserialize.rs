use rkyv::{
  check_archived_root,
  de::{deserializers::SharedDeserializeMap, SharedDeserializeRegistry, SharedPointer},
  validation::validators::DefaultValidator,
  Archive, CheckBytes, Deserialize, Fallible,
};

#[derive(Debug)]
pub enum DeserializeError {
  /// A validation error occurred
  CheckBytesError,
  /// A shared pointer was added multiple times
  DuplicateSharedPointer,
  /// A deserialize failed occurred
  DeserializeFailed(&'static str),
}

impl std::fmt::Display for DeserializeError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::CheckBytesError => {
        write!(f, "CheckBytesError")
      }
      Self::DuplicateSharedPointer => {
        write!(f, "DuplicateSharedPointer")
      }
      Self::DeserializeFailed(s) => {
        write!(f, "DeserializeFailed {}", s)
      }
    }
  }
}

impl std::error::Error for DeserializeError {}

pub struct CacheableDeserializer {
  shared: SharedDeserializeMap,
  context: *const (),
}

impl CacheableDeserializer {
  fn new<C>(context: &C) -> Self {
    Self {
      shared: SharedDeserializeMap::default(),
      context: context as *const C as *const (),
    }
  }

  // TODO change to safe implement
  pub unsafe fn context<C>(&self) -> &C {
    &*self.context.cast::<C>()
  }
}

impl Fallible for CacheableDeserializer {
  type Error = DeserializeError;
}

impl SharedDeserializeRegistry for CacheableDeserializer {
  fn get_shared_ptr(&mut self, ptr: *const u8) -> Option<&dyn SharedPointer> {
    self.shared.get_shared_ptr(ptr)
  }

  fn add_shared_ptr(
    &mut self,
    ptr: *const u8,
    shared: Box<dyn SharedPointer>,
  ) -> Result<(), Self::Error> {
    self
      .shared
      .add_shared_ptr(ptr, shared)
      .map_err(|_| DeserializeError::DuplicateSharedPointer)
  }
}

pub fn from_bytes<T, C>(bytes: &[u8], context: &C) -> Result<T, DeserializeError>
where
  T: Archive,
  T::Archived: for<'a> CheckBytes<DefaultValidator<'a>> + Deserialize<T, CacheableDeserializer>,
{
  let mut deserializer = CacheableDeserializer::new(context);
  check_archived_root::<T>(bytes)
    .map_err(|_| DeserializeError::CheckBytesError)?
    .deserialize(&mut deserializer)
}
