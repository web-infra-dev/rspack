use std::{path::Path, ptr};

use rspack_cacheable::{
  ContextGuard, Error, Result,
  rkyv::{
    Archive, Place, Serialize, SerializeUnsized,
    de::Pooling,
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
    match root.and_then(|root| strip_root_prefix(path, encoded_bytes(root), SEPARATORS)) {
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

const SEPARATORS: &[u8] = if cfg!(windows) { b"\\/" } else { b"/" };

fn encoded_bytes(path: &Path) -> &[u8] {
  path.as_os_str().as_encoded_bytes()
}

/// `root` without trailing separators, so that a stored suffix concatenates back onto any root
/// with one copy.
fn trim_trailing_separators<'a>(root: &'a [u8], separators: &[u8]) -> &'a [u8] {
  let end = root
    .iter()
    .rposition(|byte| !separators.contains(byte))
    .map_or(0, |last| last + 1);
  &root[..end]
}

fn trim_leading_separators<'a>(bytes: &'a [u8], separators: &[u8]) -> &'a [u8] {
  let start = bytes
    .iter()
    .position(|byte| !separators.contains(byte))
    .unwrap_or(bytes.len());
  &bytes[start..]
}

/// The suffix of `path` below `root`, starting at the separator that follows `root`, when `root`
/// is a whole-component prefix and the suffix has no `..` component.
///
/// Bytes are compared verbatim first. When `separators` holds more than one byte, as on Windows, a
/// path that fails the verbatim comparison is compared again with any run of separators matching
/// any other run, so that spellings `Path` treats as equal rebase alike.
pub fn strip_root_prefix<'a>(path: &'a [u8], root: &[u8], separators: &[u8]) -> Option<&'a [u8]> {
  let is_separator = |byte: &u8| separators.contains(byte);
  let prefix = trim_trailing_separators(root, separators);
  if prefix.is_empty() && (root.is_empty() || !path.first().is_some_and(is_separator)) {
    return None;
  }
  let suffix = match path.strip_prefix(prefix) {
    Some(suffix) => suffix,
    None if separators.len() > 1 => {
      strip_prefix_ignoring_separator_spelling(path, prefix, separators)?
    }
    None => return None,
  };
  if !suffix.first().is_none_or(is_separator) {
    return None;
  }
  if memchr::memmem::find(suffix, b"..").is_some() && suffix.split(is_separator).any(|c| c == b"..")
  {
    return None;
  }
  if prefix.is_empty() && suffix.iter().all(is_separator) {
    Some(&[])
  } else {
    Some(suffix)
  }
}

fn strip_prefix_ignoring_separator_spelling<'a>(
  mut path: &'a [u8],
  mut prefix: &[u8],
  separators: &[u8],
) -> Option<&'a [u8]> {
  while let Some((byte, rest)) = prefix.split_first() {
    if separators.contains(byte) {
      if !path.first().is_some_and(|byte| separators.contains(byte)) {
        return None;
      }
      path = trim_leading_separators(path, separators);
      prefix = trim_leading_separators(prefix, separators);
    } else {
      path = path.strip_prefix(&[*byte])?;
      prefix = rest;
    }
  }
  Some(path)
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
    match field.as_slice().split_first() {
      Some((&NATIVE, bytes)) => Ok(InternedPath::from_bytes(bytes)),
      Some((&UNDER_ROOT, suffix)) => {
        let guard = ContextGuard::pooling_guard(deserializer)?;
        let Some(root) = guard.project_root() else {
          return Err(Error::MessageError(
            "portable interned path restored without a project root",
          ));
        };
        let root = if suffix.is_empty() {
          encoded_bytes(root)
        } else {
          trim_trailing_separators(encoded_bytes(root), SEPARATORS)
        };
        let mut bytes = SmallVec::<[u8; 512]>::with_capacity(root.len() + suffix.len());
        bytes.extend_from_slice(root);
        bytes.extend_from_slice(suffix);
        Ok(InternedPath::from_bytes(&bytes))
      }
      _ => Err(Error::MessageError("unknown interned path payload tag")),
    }
  }
}
