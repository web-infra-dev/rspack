use std::{
  cmp::Ordering,
  fmt,
  hash::{Hash, Hasher},
};

use rspack_cacheable::{Error as CacheableError, Result as CacheableResult, cacheable};
use rspack_intern::{InternSliceStorage, InternedSlice, SliceInternable};
use smallvec::SmallVec;

use super::{
  abs_path::{AbsPath, fx_hash_bytes},
  normalize::{
    is_canonical_posix_rel_path, is_canonical_unix_abs_path, is_canonical_windows_abs_path,
    normalize_posix_rel_path, validate_root_name,
  },
};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ProjectPathHeader {
  pub(crate) hash: u64,
  /// `0` indicates `External`; `1..=254` indicates `Anchored` with `root_name.len()`.
  pub(crate) root_len: u16,
}

struct ProjectPathInner;

impl SliceInternable for ProjectPathInner {
  type Header = ProjectPathHeader;
  type Item = u8;

  #[inline]
  fn hash(header: &ProjectPathHeader, _bytes: &[u8]) -> u64 {
    header.hash
  }

  #[inline]
  fn eq(a: &[u8], b: &[u8]) -> bool {
    a == b
  }

  fn storage() -> &'static InternSliceStorage<Self> {
    static STORAGE: InternSliceStorage<ProjectPathInner> = InternSliceStorage::new();
    &STORAGE
  }
}

/// Borrowed view of a [`ProjectPath`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ProjectPathParts<'a> {
  Anchored { root: &'a str, rel: &'a str },
  External(&'a str),
}

/// An 8-byte interned, strictly UTF-8, context-free portable path for `new_cache`.
#[cacheable(with = super::cacheable::AsProjectPath)]
#[derive(Clone, PartialEq, Eq)]
pub struct ProjectPath(InternedSlice<ProjectPathInner>);

impl ProjectPath {
  pub fn anchored(root: &str, rel: &str) -> crate::Result<Self> {
    validate_root_name(root)?;
    let rel = normalize_posix_rel_path(rel)?;
    Ok(Self::intern_anchored_canonical(root, &rel))
  }

  #[inline]
  pub fn external(abs: &AbsPath) -> Self {
    Self(InternedSlice::new(
      ProjectPathHeader {
        hash: abs.precomputed_hash(),
        root_len: 0,
      },
      abs.as_str().as_bytes(),
    ))
  }

  #[inline]
  pub(crate) fn from_validated_root_and_os_rel(root: &str, os_rel: &str) -> Self {
    if os_rel.is_empty() || cfg!(not(windows)) {
      return Self::intern_anchored_canonical(root, os_rel);
    }

    let root_len = root.len();
    let total_len = root_len + 1 + os_rel.len();
    if total_len <= 512 {
      let mut buf = [0u8; 512];
      buf[..root_len].copy_from_slice(root.as_bytes());
      buf[root_len] = b'/';
      for (dst, &b) in buf[root_len + 1..total_len]
        .iter_mut()
        .zip(os_rel.as_bytes())
      {
        *dst = if b == b'\\' { b'/' } else { b };
      }
      let slice = &buf[..total_len];
      return Self(InternedSlice::new(
        ProjectPathHeader {
          hash: fx_hash_bytes(slice),
          root_len: root_len as u16,
        },
        slice,
      ));
    }

    let mut buf = Vec::with_capacity(total_len);
    buf.extend_from_slice(root.as_bytes());
    buf.push(b'/');
    buf.extend(
      os_rel
        .as_bytes()
        .iter()
        .map(|&b| if b == b'\\' { b'/' } else { b }),
    );
    Self(InternedSlice::new(
      ProjectPathHeader {
        hash: fx_hash_bytes(&buf),
        root_len: root_len as u16,
      },
      &buf,
    ))
  }

  #[inline]
  pub(crate) fn external_from_normalized_str(normalized: &str) -> Self {
    let bytes = normalized.as_bytes();
    Self(InternedSlice::new(
      ProjectPathHeader {
        hash: fx_hash_bytes(bytes),
        root_len: 0,
      },
      bytes,
    ))
  }

  pub(crate) fn from_wire_bytes(bytes: &[u8]) -> CacheableResult<Self> {
    let Some((&tag, payload)) = bytes.split_first() else {
      return Err(CacheableError::MessageError("empty archived project path"));
    };
    let s = std::str::from_utf8(payload)
      .map_err(|_| CacheableError::MessageError("invalid UTF-8 in archived project path"))?;

    if tag == 0 {
      if !is_canonical_unix_abs_path(s) && !is_canonical_windows_abs_path(s) {
        return Err(CacheableError::MessageError(
          "invalid external archived project path",
        ));
      }
      return Ok(Self::external_from_normalized_str(s));
    }

    if tag > 254 {
      return Err(CacheableError::MessageError(
        "invalid archived project path tag",
      ));
    }

    let root_len = usize::from(tag);
    if payload.len() < root_len || !s.is_char_boundary(root_len) {
      return Err(CacheableError::MessageError(
        "invalid root_len in archived project path",
      ));
    }
    let root = &s[..root_len];
    if validate_root_name(root).is_err() {
      return Err(CacheableError::MessageError(
        "invalid root name in archived project path",
      ));
    }
    if payload.len() > root_len
      && (payload[root_len] != b'/' || !is_canonical_posix_rel_path(&s[root_len + 1..]))
    {
      return Err(CacheableError::MessageError(
        "invalid relative segment in archived project path",
      ));
    }

    Ok(Self(InternedSlice::new(
      ProjectPathHeader {
        hash: fx_hash_bytes(payload),
        root_len: u16::from(tag),
      },
      payload,
    )))
  }

  #[inline]
  pub(crate) fn wire_tag(&self) -> u8 {
    self.0.header().root_len as u8
  }

  #[inline]
  pub(crate) fn raw_items(&self) -> &[u8] {
    self.0.items()
  }

  #[inline]
  pub(crate) fn header_addr(&self) -> usize {
    std::ptr::from_ref(self.0.header()).addr()
  }

  #[inline]
  pub fn parts(&self) -> ProjectPathParts<'_> {
    // SAFETY: `ProjectPath` only ever interns valid UTF-8 bytes.
    let s = unsafe { std::str::from_utf8_unchecked(self.0.items()) };
    let root_len = usize::from(self.0.header().root_len);
    if root_len == 0 {
      ProjectPathParts::External(s)
    } else {
      let rel = if s.len() == root_len {
        ""
      } else {
        &s[root_len + 1..]
      };
      ProjectPathParts::Anchored {
        root: &s[..root_len],
        rel,
      }
    }
  }

  #[inline]
  pub fn is_external(&self) -> bool {
    self.0.header().root_len == 0
  }

  #[inline]
  pub fn is_anchored(&self) -> bool {
    self.0.header().root_len != 0
  }

  #[inline]
  pub fn precomputed_hash(&self) -> u64 {
    self.0.header().hash
  }

  pub fn try_join(&self, sub: &str) -> Option<Self> {
    let ProjectPathParts::Anchored { root, rel } = self.parts() else {
      return None;
    };
    if sub.is_empty() {
      return Some(self.clone());
    }
    let bytes = sub.as_bytes();
    if cfg!(windows) {
      if matches!(bytes[0], b'/' | b'\\')
        || (bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':')
      {
        return None;
      }
    } else if bytes[0] == b'/' || sub.contains('\\') {
      return None;
    }

    if is_canonical_posix_rel_path(sub) {
      return Some(self.append_clean_posix_rel(sub));
    }

    let mut stack: SmallVec<[&str; 16]> = if rel.is_empty() {
      SmallVec::new()
    } else {
      rel.split('/').collect()
    };

    let mut changed = false;
    let seps: &[char] = if cfg!(windows) { &['/', '\\'] } else { &['/'] };
    for seg in sub.split(seps) {
      match seg {
        "" | "." => {}
        ".." => {
          stack.pop()?;
          changed = true;
        }
        _ => {
          stack.push(seg);
          changed = true;
        }
      }
    }

    if !changed {
      return Some(self.clone());
    }
    if stack.is_empty() {
      let root_len = self.0.header().root_len;
      let slice = &self.0.items()[..usize::from(root_len)];
      return Some(Self(InternedSlice::new(
        ProjectPathHeader {
          hash: fx_hash_bytes(slice),
          root_len,
        },
        slice,
      )));
    }
    Some(Self::intern_anchored_canonical(root, &stack.join("/")))
  }

  pub fn parent_in_root(&self) -> Option<Self> {
    let root_len = self.0.header().root_len;
    if root_len == 0 {
      return None;
    }
    let items = self.0.items();
    if items.len() <= usize::from(root_len) {
      return None;
    }
    // SAFETY: `ProjectPath` only ever interns valid UTF-8 bytes.
    let s = unsafe { std::str::from_utf8_unchecked(items) };
    let idx = s.rfind('/')?;
    let slice = &items[..idx];
    Some(Self(InternedSlice::new(
      ProjectPathHeader {
        hash: fx_hash_bytes(slice),
        root_len,
      },
      slice,
    )))
  }

  #[inline]
  fn intern_anchored_canonical(root: &str, rel: &str) -> Self {
    let root_len = root.len();
    if rel.is_empty() {
      let bytes = root.as_bytes();
      return Self(InternedSlice::new(
        ProjectPathHeader {
          hash: fx_hash_bytes(bytes),
          root_len: root_len as u16,
        },
        bytes,
      ));
    }

    let total_len = root_len + 1 + rel.len();
    if total_len <= 512 {
      let mut buf = [0u8; 512];
      buf[..root_len].copy_from_slice(root.as_bytes());
      buf[root_len] = b'/';
      buf[root_len + 1..total_len].copy_from_slice(rel.as_bytes());
      let slice = &buf[..total_len];
      return Self(InternedSlice::new(
        ProjectPathHeader {
          hash: fx_hash_bytes(slice),
          root_len: root_len as u16,
        },
        slice,
      ));
    }

    let mut buf = Vec::with_capacity(total_len);
    buf.extend_from_slice(root.as_bytes());
    buf.push(b'/');
    buf.extend_from_slice(rel.as_bytes());
    Self(InternedSlice::new(
      ProjectPathHeader {
        hash: fx_hash_bytes(&buf),
        root_len: root_len as u16,
      },
      &buf,
    ))
  }

  #[inline]
  fn append_clean_posix_rel(&self, suffix: &str) -> Self {
    let items = self.0.items();
    let root_len = self.0.header().root_len;
    let total_len = items.len() + 1 + suffix.len();
    if total_len <= 512 {
      let mut buf = [0u8; 512];
      buf[..items.len()].copy_from_slice(items);
      buf[items.len()] = b'/';
      buf[items.len() + 1..total_len].copy_from_slice(suffix.as_bytes());
      let slice = &buf[..total_len];
      return Self(InternedSlice::new(
        ProjectPathHeader {
          hash: fx_hash_bytes(slice),
          root_len,
        },
        slice,
      ));
    }

    let mut buf = Vec::with_capacity(total_len);
    buf.extend_from_slice(items);
    buf.push(b'/');
    buf.extend_from_slice(suffix.as_bytes());
    Self(InternedSlice::new(
      ProjectPathHeader {
        hash: fx_hash_bytes(&buf),
        root_len,
      },
      &buf,
    ))
  }
}

impl fmt::Debug for ProjectPath {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self.parts() {
      ProjectPathParts::Anchored { root, rel: "" } => write!(f, "[{root}]"),
      ProjectPathParts::Anchored { root, rel } => write!(f, "[{root}]/{rel}"),
      ProjectPathParts::External(abs) => f.write_str(abs),
    }
  }
}

impl Hash for ProjectPath {
  #[inline]
  fn hash<H: Hasher>(&self, state: &mut H) {
    state.write_u64(self.precomputed_hash());
  }
}

impl PartialOrd for ProjectPath {
  #[inline]
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for ProjectPath {
  #[inline]
  fn cmp(&self, other: &Self) -> Ordering {
    match (self.parts(), other.parts()) {
      (
        ProjectPathParts::Anchored { root: r1, rel: p1 },
        ProjectPathParts::Anchored { root: r2, rel: p2 },
      ) => r1.cmp(r2).then_with(|| p1.cmp(p2)),
      (ProjectPathParts::Anchored { .. }, ProjectPathParts::External(_)) => Ordering::Less,
      (ProjectPathParts::External(_), ProjectPathParts::Anchored { .. }) => Ordering::Greater,
      (ProjectPathParts::External(a), ProjectPathParts::External(b)) => a.cmp(b),
    }
  }
}
