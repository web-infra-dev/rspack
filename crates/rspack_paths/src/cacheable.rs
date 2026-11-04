use std::{path::Path, ptr};

use rspack_cacheable::{
  ContextGuard, Error, Result, RootSpelling,
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
const FIXED: u8 = 6;
const UNDER_ROOT: u8 = 1;
const UNDER_REALPATH: u8 = 2;
/// Followed by the number of ancestors above the root as one byte, then the bytes below that
/// ancestor. Resolver lookups walk up past the root, so a worktree's `../node_modules` misses
/// would otherwise bind every module they belong to. The walk may reach the filesystem root:
/// the cache identity ties both roots to one filesystem, so `/package.json` restores as itself.
const ABOVE_ROOT: u8 = 3;
const ABOVE_REALPATH: u8 = 4;
/// Followed by the writer's root depth and the bytes below its filesystem root. The path
/// stays anchored at that root. Recording depth makes snapshot keys fail validation when a move
/// introduces ancestors that the writer never searched. One tag covers both root spellings.
const AT_FS_ROOT: u8 = 5;

/// One distinct path as written: a tag byte followed by `OsStr::as_encoded_bytes`.
enum PathPayload<'a> {
  /// The complete native bytes.
  Native(&'a [u8]),
  Fixed(&'a [u8]),
  /// The bytes below a spelling of the portable root, starting at the separator that follows it.
  UnderRoot(RootSpelling, &'a [u8]),
  /// The bytes below the `up`th ancestor of a spelling of the portable root.
  AboveRoot(RootSpelling, u8, &'a [u8]),
  /// The bytes below the filesystem root that holds the portable root.
  AtFsRoot(u8, &'a [u8]),
}

impl<'a> PathPayload<'a> {
  fn new(path: &'a [u8], guard: &ContextGuard) -> Self {
    let spellings: SmallVec<[(RootSpelling, &Path); 2]> = guard.root_spellings().collect();
    if let Some(payload) = spellings.iter().find_map(|&(spelling, root)| {
      strip_root_prefix(path, encoded_bytes(root), SEPARATORS)
        .map(|suffix| Self::UnderRoot(spelling, suffix))
    }) {
      return payload;
    }
    if !guard.allows_above_root() {
      return Self::Native(path);
    }
    for &(spelling, root) in &spellings {
      let mut ancestor = root;
      // Walk to the filesystem root. The cache identity ties both roots to one filesystem, so
      // every ancestor restores by layout; only a depth beyond a byte stays native.
      for up in 1..=usize::from(u8::MAX) {
        let Some(parent) = ancestor.parent() else {
          break;
        };
        ancestor = parent;
        let Some(suffix) = strip_root_prefix(path, encoded_bytes(ancestor), SEPARATORS)
          .filter(|suffix| !suffix.is_empty())
        else {
          continue;
        };
        if ancestor.parent().is_none() {
          return Self::AtFsRoot(
            u8::try_from(up).expect("the loop bound keeps depth within a byte"),
            suffix,
          );
        }
        return Self::AboveRoot(
          spelling,
          u8::try_from(up).expect("the loop bound keeps depth within a byte"),
          suffix,
        );
      }
    }
    Self::Native(path)
  }

  fn parts(&self) -> (u8, Option<u8>, &'a [u8]) {
    match *self {
      Self::Native(bytes) => (NATIVE, None, bytes),
      Self::Fixed(bytes) => (FIXED, None, bytes),
      Self::UnderRoot(RootSpelling::Context, bytes) => (UNDER_ROOT, None, bytes),
      Self::UnderRoot(RootSpelling::Realpath, bytes) => (UNDER_REALPATH, None, bytes),
      Self::AboveRoot(RootSpelling::Context, up, bytes) => (ABOVE_ROOT, Some(up), bytes),
      Self::AboveRoot(RootSpelling::Realpath, up, bytes) => (ABOVE_REALPATH, Some(up), bytes),
      Self::AtFsRoot(depth, bytes) => (AT_FS_ROOT, Some(depth), bytes),
    }
  }
}

/// Walks above `root`, clamping at its filesystem root when the reader is shallower.
/// Native path components preserve Windows drive and UNC share boundaries.
fn ancestor_bytes(root: &Path, up: u8) -> &[u8] {
  let mut ancestor = root;
  for _ in 0..up {
    match ancestor.parent() {
      Some(parent) => ancestor = parent,
      None => break,
    }
  }
  let bytes = encoded_bytes(ancestor);
  let trimmed = trim_trailing_separators(bytes, SEPARATORS);
  if trimmed.is_empty() { bytes } else { trimmed }
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
    let (_, up, bytes) = self.parts();
    ArchivedVec::resolve_from_len(bytes.len() + 1 + usize::from(up.is_some()), resolver, out);
  }
}

impl<S> Serialize<S> for PathPayload<'_>
where
  S: Fallible<Error = Error> + Writer + ?Sized,
{
  fn serialize(&self, serializer: &mut S) -> Result<Self::Resolver> {
    let (tag, up, bytes) = self.parts();
    let pos = serializer.pos();
    serializer.write(&[tag])?;
    if let Some(up) = up {
      serializer.write(&[up])?;
    }
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
    let guard = ContextGuard::sharing_guard(serializer)?;
    let payload = if field.is_fixed() {
      PathPayload::Fixed(field.0.items())
    } else {
      PathPayload::new(field.0.items(), guard)
    };
    if guard.project_root().is_some()
      && (field.is_fixed()
        || (matches!(payload, PathPayload::Native(_)) && field.as_ref().is_absolute()))
    {
      guard.mark_outside_root();
    }
    // Sharing is scoped to the encoding policy as well as the interned identity. A fixed path
    // must never reuse root-relative bytes, nor may an ordinary path inherit above-root authority.
    // The header is a u64, leaving these two alignment bits available.
    let policy = if field.is_fixed() {
      1
    } else if guard.allows_above_root() {
      2
    } else {
      0
    };
    let address = ptr::from_ref(field.0.header()).addr() | policy;
    let pos = match serializer.start_sharing(address) {
      SharingState::Started => {
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
      Some((&FIXED, bytes)) => Ok(InternedPath::from_bytes(bytes).into_fixed()),
      Some((
        &tag @ (UNDER_ROOT | UNDER_REALPATH | ABOVE_ROOT | ABOVE_REALPATH | AT_FS_ROOT),
        rest,
      )) => {
        let guard = ContextGuard::pooling_guard(deserializer)?;
        let spelling = if matches!(tag, UNDER_ROOT | ABOVE_ROOT | AT_FS_ROOT) {
          RootSpelling::Context
        } else {
          RootSpelling::Realpath
        };
        let Some(root) = guard.root(spelling) else {
          return Err(Error::MessageError(
            "portable interned path restored without a project root",
          ));
        };
        let (root, suffix) = if tag == AT_FS_ROOT {
          let Some((_, rest)) = rest.split_first() else {
            return Err(Error::MessageError(
              "interned path at filesystem root has no depth",
            ));
          };
          let fs_root = ancestor_bytes(root.ancestors().last().unwrap_or(root), 0);
          if trim_trailing_separators(fs_root, SEPARATORS).is_empty() {
            (fs_root, trim_leading_separators(rest, SEPARATORS))
          } else {
            (fs_root, rest)
          }
        } else if matches!(tag, ABOVE_ROOT | ABOVE_REALPATH) {
          let Some((&up, suffix)) = rest.split_first() else {
            return Err(Error::MessageError(
              "interned path above the root has no depth",
            ));
          };
          let ancestor = ancestor_bytes(root, up);
          // A bare `/` already ends in the separator the suffix starts with.
          if trim_trailing_separators(ancestor, SEPARATORS).is_empty() {
            (ancestor, trim_leading_separators(suffix, SEPARATORS))
          } else {
            (ancestor, suffix)
          }
        } else if rest.is_empty() {
          (encoded_bytes(root), rest)
        } else {
          (
            trim_trailing_separators(encoded_bytes(root), SEPARATORS),
            rest,
          )
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
