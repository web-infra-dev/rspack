use std::{path::Path, ptr};

use rspack_cacheable::{
  ContextGuard, Error, Result,
  rkyv::{
    Archive, Place, Serialize, SerializeUnsized,
    de::{ErasedPtr, Pooling, PoolingState},
    rancor::Fallible,
    rc::{ArchivedRc, Flavor, RcResolver},
    ser::{Allocator, Positional, Sharing, Writer, sharing::SharingState},
    vec::{ArchivedVec, VecResolver},
    with::{ArchiveWith, DeserializeWith, SerializeWith},
  },
};
use smallvec::SmallVec;

use crate::InternedPath;

/// Shares path bytes within an archive and rebases paths under the portable root.
pub struct AsInternedPath;

type ArchivedInternedPath = ArchivedRc<ArchivedVec<u8>, AsInternedPath>;

const NATIVE: u8 = 0;
const UNDER_ROOT: u8 = 1;

/// One distinct path as written: a tag byte followed by `OsStr::as_encoded_bytes`.
enum PathPayload<'a> {
  /// The complete native bytes.
  Native(&'a [u8]),
  /// The bytes below the portable root, starting at the separator that follows the root.
  UnderRoot(&'a [u8]),
}

impl<'a> PathPayload<'a> {
  fn new(path: &'a [u8], root: Option<&Path>) -> Self {
    match root.and_then(|root| strip_root(path, root)) {
      Some(suffix) => Self::UnderRoot(suffix),
      None => Self::Native(path),
    }
  }

  fn parts(&self) -> (u8, &'a [u8]) {
    match *self {
      Self::Native(bytes) => (NATIVE, bytes),
      Self::UnderRoot(bytes) => (UNDER_ROOT, bytes),
    }
  }
}

/// The root's encoded bytes without a trailing separator, so that a stored suffix concatenates
/// back onto any root with one copy.
fn root_prefix(root: &Path) -> &[u8] {
  let bytes = root.as_os_str().as_encoded_bytes();
  let end = bytes
    .iter()
    .rposition(|&byte| !is_separator(byte))
    .map_or(0, |last| last + 1);
  &bytes[..end]
}

/// The suffix of `path` below `root`, when `root` is a whole-component prefix and the suffix
/// contains no parent components.
fn strip_root<'a>(path: &'a [u8], root: &Path) -> Option<&'a [u8]> {
  let prefix = root_prefix(root);
  if prefix.is_empty()
    && (!root.is_absolute() || !path.first().is_some_and(|&byte| is_separator(byte)))
  {
    return None;
  }
  let suffix = path.strip_prefix(prefix)?;
  if !suffix.first().is_none_or(|&byte| is_separator(byte)) {
    return None;
  }
  if memchr::memmem::find(suffix, b"..").is_some()
    && suffix.split(|&byte| is_separator(byte)).any(|c| c == b"..")
  {
    return None;
  }
  if prefix.is_empty() && suffix.iter().all(|&byte| is_separator(byte)) {
    Some(&[])
  } else {
    Some(suffix)
  }
}

fn is_separator(byte: u8) -> bool {
  std::path::is_separator(byte as char)
}

impl Archive for PathPayload<'_> {
  type Archived = ArchivedVec<u8>;
  type Resolver = VecResolver;

  fn resolve(&self, resolver: Self::Resolver, out: Place<Self::Archived>) {
    ArchivedVec::resolve_from_len(self.parts().1.len() + 1, resolver, out);
  }
}

impl<S> Serialize<S> for PathPayload<'_>
where
  S: Fallible<Error = Error> + Writer + ?Sized,
{
  fn serialize(&self, serializer: &mut S) -> Result<Self::Resolver> {
    let (tag, bytes) = self.parts();
    let pos = serializer.pos();
    serializer.write(&[tag])?;
    serializer.write(bytes)?;
    Ok(VecResolver::from_pos(pos))
  }
}

impl Flavor for AsInternedPath {
  const ALLOW_CYCLES: bool = false;
}

impl ArchiveWith<InternedPath> for AsInternedPath {
  type Archived = ArchivedInternedPath;
  type Resolver = RcResolver;

  #[inline]
  fn resolve_with(_field: &InternedPath, resolver: Self::Resolver, out: Place<Self::Archived>) {
    // The payload is sized, so only the resolver's position is needed.
    ArchivedRc::resolve_from_ref(&PathPayload::Native(&[]), resolver, out);
  }
}

impl<S> SerializeWith<InternedPath, S> for AsInternedPath
where
  S: Fallible<Error = Error> + Writer + Allocator + Sharing + ?Sized,
{
  #[inline]
  fn serialize_with(field: &InternedPath, serializer: &mut S) -> Result<Self::Resolver> {
    // The interned header provides a stable sharing key, including for empty paths.
    let address = ptr::from_ref(field.0.header()).addr();
    let pos = match serializer.start_sharing(address) {
      SharingState::Started => {
        let guard = ContextGuard::sharing_guard(serializer)?;
        let payload = PathPayload::new(field.0.items(), guard.project_root());
        if matches!(payload, PathPayload::Native(_))
          && guard.project_root().is_some()
          && field.as_ref().is_absolute()
        {
          guard.mark_outside_root();
        }
        let pos = payload.serialize_unsized(serializer)?;
        serializer.finish_sharing(address, pos)?;
        pos
      }
      SharingState::Finished(pos) => pos,
      SharingState::Pending => {
        return Err(Error::MessageError("cyclic shared interned path"));
      }
    };
    Ok(RcResolver::from_pos(pos))
  }
}

impl<D> DeserializeWith<ArchivedInternedPath, InternedPath, D> for AsInternedPath
where
  D: Fallible<Error = Error> + Pooling + ?Sized,
{
  #[inline]
  fn deserialize_with(field: &ArchivedInternedPath, deserializer: &mut D) -> Result<InternedPath> {
    let address = ptr::from_ref(&**field).addr();
    match deserializer.start_pooling(address) {
      PoolingState::Finished(ptr) => {
        // SAFETY: this address is registered below with a boxed InternedPath.
        return Ok(unsafe { &*ptr.data_address().cast::<InternedPath>() }.clone());
      }
      PoolingState::Pending => return Err(Error::MessageError("cyclic shared interned path")),
      PoolingState::Started => {}
    }
    let guard = ContextGuard::pooling_guard(deserializer)?;
    let path = match field.as_slice().split_first() {
      Some((&NATIVE, bytes)) => Ok(InternedPath::from_bytes(bytes)),
      Some((&UNDER_ROOT, suffix)) => {
        let Some(root) = guard.project_root() else {
          return Err(Error::MessageError(
            "portable interned path restored without a project root",
          ));
        };
        let root = if suffix.is_empty() {
          root.as_os_str().as_encoded_bytes()
        } else {
          root_prefix(root)
        };
        let mut bytes = SmallVec::<[u8; 512]>::with_capacity(root.len() + suffix.len());
        bytes.extend_from_slice(root);
        bytes.extend_from_slice(suffix);
        Ok(InternedPath::from_bytes(&bytes))
      }
      _ => Err(Error::MessageError("unknown interned path payload tag")),
    }?;
    let mut pooled = Box::new(path.clone());
    // SAFETY: the pool owns this Box on success and releases it with drop_path.
    unsafe {
      deserializer.finish_pooling(
        address,
        ErasedPtr::new(ptr::from_mut(&mut *pooled)),
        drop_path,
      )?;
    }
    let _ = Box::into_raw(pooled);
    Ok(path)
  }
}

unsafe fn drop_path(ptr: ErasedPtr) {
  // SAFETY: finish_pooling receives only Box<InternedPath> pointers with this drop function.
  drop(unsafe { Box::from_raw(ptr.data_address().cast::<InternedPath>()) });
}
