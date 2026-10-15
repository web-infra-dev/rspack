use std::{
  borrow::Cow,
  fmt,
  hash::{Hash, Hasher},
  marker::PhantomData,
  ops::{Deref, Range},
  ptr::{self, NonNull},
  slice, str,
  sync::atomic::{AtomicU32, Ordering},
};

use serde::{Serialize, Serializer};

use crate::helpers::utf16_len;

const STATUS_MASK: u32 = 0b11;
const UNKNOWN: u32 = 0;
const ASCII: u32 = 1;
const NON_ASCII: u32 = 2;
const MODE_MASK: u32 = 0b11 << 30;
const LOCAL: u32 = 0;
const OWNER: u32 = 0b01 << 30;
const SHARED: u32 = 0b10 << 30;

#[repr(C)]
struct ContentRepr {
  // LOCAL and OWNER point to UTF-8 bytes. SHARED points to another ContentRepr.
  ptr: NonNull<u8>,
  len: u32,
  tag: AtomicU32,
}

impl ContentRepr {
  #[inline]
  fn mode(&self) -> u32 {
    self.tag.load(Ordering::Relaxed) & MODE_MASK
  }

  #[inline]
  fn target(&self) -> &ContentRepr {
    if self.mode() == SHARED {
      // SAFETY: A shared view is created only from a live borrowed ContentRepr, and
      // always points directly to a LOCAL or OWNER ContentRepr.
      unsafe { self.ptr.cast::<ContentRepr>().as_ref() }
    } else {
      self
    }
  }

  fn as_bytes(&self) -> &[u8] {
    let target = self.target();
    // SAFETY: All owners hold immutable Box<str> bytes, and local views are
    // constructed from a valid borrowed str. Their lifetimes outlive this view.
    unsafe { slice::from_raw_parts(target.ptr.as_ptr(), target.len as usize) }
  }

  #[inline]
  fn status(&self) -> u32 {
    self.target().tag.load(Ordering::Relaxed) & STATUS_MASK
  }

  fn is_ascii(&self) -> bool {
    let target = self.target();
    let current = target.tag.load(Ordering::Relaxed);
    match current & STATUS_MASK {
      ASCII => true,
      NON_ASCII => false,
      UNKNOWN => {
        let ascii = target.as_bytes().is_ascii();
        // Concurrent scans compute the same answer from immutable bytes.
        target.tag.store(
          (current & MODE_MASK) | if ascii { ASCII } else { NON_ASCII },
          Ordering::Relaxed,
        );
        ascii
      }
      _ => unreachable!(),
    }
  }
}

/// An owned UTF-8 string with a lazily cached ASCII result.
///
/// Cloning duplicates the bytes and their current ASCII result.
#[repr(transparent)]
pub struct Content {
  repr: ContentRepr,
}

impl Content {
  /// Concatenate borrowed chunks and cache whether the result is ASCII.
  ///
  /// `capacity` is the total UTF-8 byte length when known.
  pub fn from_iter<'a>(
    chunks: impl IntoIterator<Item = ContentSlice<'a>>,
    capacity: usize,
  ) -> Self {
    let mut text = String::with_capacity(capacity);
    let mut is_ascii = true;
    for chunk in chunks {
      if is_ascii {
        is_ascii = chunk.is_ascii();
      }
      text.push_str(chunk.as_str());
    }
    let value = Self::from(text);
    value.repr.tag.store(
      OWNER | if is_ascii { ASCII } else { NON_ASCII },
      Ordering::Relaxed,
    );
    value
  }

  /// Take ownership of a boxed string without reallocating it.
  pub fn try_from_boxed(text: Box<str>) -> Result<Self, Box<str>> {
    let Ok(len) = u32::try_from(text.len()) else {
      return Err(text);
    };
    // SAFETY: Box<str> always has a non-null data pointer, including when empty.
    let ptr = unsafe { NonNull::new_unchecked(Box::into_raw(text) as *mut u8) };
    Ok(Self {
      repr: ContentRepr {
        ptr,
        len,
        tag: AtomicU32::new(OWNER | UNKNOWN),
      },
    })
  }

  /// Take ownership of a boxed string, panicking if it exceeds u32::MAX bytes.
  pub fn from_boxed(text: Box<str>) -> Self {
    Self::try_from_boxed(text).unwrap_or_else(|_| panic!("content exceeds u32::MAX bytes"))
  }

  /// Borrow the same bytes and ASCII cache as this string.
  pub fn view(&self) -> ContentSlice<'_> {
    ContentSlice {
      repr: ContentRepr {
        ptr: NonNull::from(&self.repr).cast(),
        len: self.repr.len,
        tag: AtomicU32::new(SHARED),
      },
      lifetime: PhantomData,
    }
  }

  pub fn as_str(&self) -> &str {
    // SAFETY: Content stores immutable UTF-8 bytes from a boxed string.
    unsafe { str::from_utf8_unchecked(self.repr.as_bytes()) }
  }

  pub fn is_ascii(&self) -> bool {
    self.repr.is_ascii()
  }

  /// Return the cached result without scanning the text.
  pub fn cached_is_ascii(&self) -> Option<bool> {
    match self.repr.status() {
      ASCII => Some(true),
      NON_ASCII => Some(false),
      _ => None,
    }
  }

  pub fn slice(&self, range: Range<usize>) -> Option<ContentSlice<'_>> {
    self.view().slice(range)
  }

  pub fn len(&self) -> usize {
    self.repr.len as usize
  }

  pub fn is_empty(&self) -> bool {
    self.repr.len == 0
  }

  /// Return the owned UTF-8 allocation without copying its bytes.
  pub fn into_boxed_str(self) -> Box<str> {
    let this = std::mem::ManuallyDrop::new(self);
    let raw = ptr::slice_from_raw_parts_mut(this.repr.ptr.as_ptr(), this.repr.len as usize);
    // SAFETY: Content owns exactly this Box<str> allocation.
    unsafe { Box::from_raw(raw as *mut str) }
  }

  /// Return a String without copying its bytes.
  pub fn into_string(self) -> String {
    self.into_boxed_str().into()
  }

  /// Return the UTF-8 bytes without copying them.
  pub fn into_bytes(self) -> Vec<u8> {
    self.into_string().into_bytes()
  }
}

impl Default for Content {
  fn default() -> Self {
    Self::from_boxed(Box::<str>::default())
  }
}

impl From<Box<str>> for Content {
  fn from(value: Box<str>) -> Self {
    Self::from_boxed(value)
  }
}

impl From<String> for Content {
  fn from(value: String) -> Self {
    Self::from_boxed(value.into_boxed_str())
  }
}

impl From<&str> for Content {
  fn from(value: &str) -> Self {
    Self::from_boxed(value.into())
  }
}

impl From<Cow<'_, str>> for Content {
  fn from(value: Cow<'_, str>) -> Self {
    match value {
      Cow::Borrowed(value) => Self::from(value),
      Cow::Owned(value) => Self::from(value),
    }
  }
}

impl Clone for Content {
  fn clone(&self) -> Self {
    let cloned = Self::from_boxed(self.as_str().into());
    cloned
      .repr
      .tag
      .store(OWNER | self.repr.status(), Ordering::Relaxed);
    cloned
  }
}

impl Drop for Content {
  fn drop(&mut self) {
    let raw = ptr::slice_from_raw_parts_mut(self.repr.ptr.as_ptr(), self.repr.len as usize);
    // SAFETY: This is the exact pointer and length taken from Box<str>.
    unsafe { drop(Box::from_raw(raw)) }
  }
}

impl Deref for Content {
  type Target = str;

  fn deref(&self) -> &Self::Target {
    self.as_str()
  }
}

impl AsRef<str> for Content {
  fn as_ref(&self) -> &str {
    self.as_str()
  }
}

impl PartialEq for Content {
  fn eq(&self, other: &Self) -> bool {
    self.as_str() == other.as_str()
  }
}
impl Eq for Content {}
impl Hash for Content {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.as_str().hash(state);
  }
}
impl fmt::Debug for Content {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.as_str().fmt(f)
  }
}
impl Serialize for Content {
  fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    self.as_str().serialize(serializer)
  }
}

#[cfg(feature = "allocative")]
impl allocative::Allocative for Content {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut allocative::Visitor<'b>) {
    let mut visitor = visitor.enter_self_sized::<Self>();
    let mut allocation = visitor.enter_unique(
      allocative::Key::new("ptr"),
      std::mem::size_of::<*const u8>(),
    );
    allocation.visit_simple(allocative::Key::new("bytes"), self.len());
    allocation.exit();
    visitor.exit();
  }
}

/// A borrowed UTF-8 string with either a shared or local ASCII cache.
///
/// A local view's clone has an independent cache; a shared view's clone keeps
/// sharing its target cache.
#[repr(transparent)]
pub struct ContentSlice<'a> {
  repr: ContentRepr,
  lifetime: PhantomData<&'a str>,
}

impl<'a> ContentSlice<'a> {
  /// Borrow an ordinary string with a local ASCII cache.
  pub fn try_from_str(text: &'a str) -> Option<Self> {
    let len = u32::try_from(text.len()).ok()?;
    // SAFETY: A str reference has a non-null pointer, including when empty.
    let ptr = unsafe { NonNull::new_unchecked(text.as_ptr() as *mut u8) };
    Some(Self {
      repr: ContentRepr {
        ptr,
        len,
        tag: AtomicU32::new(LOCAL | if text.is_empty() { ASCII } else { UNKNOWN }),
      },
      lifetime: PhantomData,
    })
  }

  #[allow(clippy::should_implement_trait)]
  pub fn from_str(text: &'a str) -> Self {
    Self::try_from_str(text).expect("content exceeds u32::MAX bytes")
  }

  pub(crate) fn with_ascii_hint(text: &'a str, is_ascii: bool) -> Self {
    debug_assert!(!is_ascii || text.is_ascii());
    let value = Self::from_str(text);
    if is_ascii {
      value.repr.tag.store(ASCII, Ordering::Relaxed);
    }
    value
  }

  pub fn as_str(&self) -> &'a str {
    let target = self.repr.target();
    // SAFETY: LOCAL borrows a str for 'a, OWNER is borrowed from Content
    // for 'a, and SHARED points to a ContentRepr borrowed for 'a.
    unsafe {
      str::from_utf8_unchecked(slice::from_raw_parts(
        target.ptr.as_ptr(),
        target.len as usize,
      ))
    }
  }

  pub fn is_ascii(&self) -> bool {
    self.repr.is_ascii()
  }

  /// Return the cached result without scanning the text.
  pub fn cached_is_ascii(&self) -> Option<bool> {
    match self.repr.status() {
      ASCII => Some(true),
      NON_ASCII => Some(false),
      _ => None,
    }
  }

  pub fn len(&self) -> usize {
    self.repr.len as usize
  }

  pub fn is_empty(&self) -> bool {
    self.repr.len == 0
  }

  #[inline]
  pub(crate) fn utf16_len(&self) -> usize {
    self.utf16_len_of(self.as_str())
  }

  #[inline]
  pub(crate) fn utf16_len_of(&self, text: &str) -> usize {
    if self.cached_is_ascii() == Some(true) {
      text.len()
    } else {
      utf16_len(text)
    }
  }

  #[inline]
  pub(crate) fn is_known_ascii(&self) -> bool {
    self.cached_is_ascii() == Some(true)
  }

  #[inline]
  pub(crate) fn subspan(&self, text: &'a str) -> Self {
    let target = self.repr.target();
    debug_assert!({
      let base = target.ptr.as_ptr() as usize;
      let start = text.as_ptr() as usize;
      start >= base
        && start - base <= target.len as usize
        && text.len() <= target.len as usize - (start - base)
    });
    if text.as_ptr() == target.ptr.as_ptr() && text.len() == target.len as usize {
      return self.clone();
    }
    // SAFETY: text is borrowed for 'a and therefore outlives this view.
    let ptr = unsafe { NonNull::new_unchecked(text.as_ptr() as *mut u8) };
    let initial = if text.is_empty() || target.tag.load(Ordering::Relaxed) & STATUS_MASK == ASCII {
      ASCII
    } else {
      UNKNOWN
    };
    Self {
      repr: ContentRepr {
        ptr,
        len: text.len() as u32,
        tag: AtomicU32::new(LOCAL | initial),
      },
      lifetime: PhantomData,
    }
  }

  /// Borrow the same bytes and cache as this view.
  pub fn share(&self) -> ContentSlice<'_> {
    let target = self.repr.target();
    ContentSlice {
      repr: ContentRepr {
        ptr: NonNull::from(target).cast(),
        len: target.len,
        tag: AtomicU32::new(SHARED),
      },
      lifetime: PhantomData,
    }
  }

  /// Return a UTF-8 byte-range view. A proper substring has its own cache.
  pub fn slice(&self, range: Range<usize>) -> Option<ContentSlice<'a>> {
    let text = self.as_str();
    let sub = text.get(range.clone())?;
    if range.start == 0 && range.end == text.len() {
      return Some(self.clone());
    }
    let len = sub.len() as u32;
    // SAFETY: sub is a valid str and lives as long as this borrowed view.
    let ptr = unsafe { NonNull::new_unchecked(sub.as_ptr() as *mut u8) };
    let initial = if sub.is_empty() || self.repr.status() == ASCII {
      ASCII
    } else {
      UNKNOWN
    };
    Some(ContentSlice {
      repr: ContentRepr {
        ptr,
        len,
        tag: AtomicU32::new(LOCAL | initial),
      },
      lifetime: PhantomData,
    })
  }

  pub(crate) fn slice_to(&self, end: usize) -> ContentSlice<'a> {
    self
      .slice(0..end)
      .expect("slice must end at a UTF-8 boundary")
  }

  pub(crate) fn slice_from(&self, start: usize) -> ContentSlice<'a> {
    self
      .slice(start..self.len())
      .expect("slice must start at a UTF-8 boundary")
  }
}

impl Clone for ContentSlice<'_> {
  fn clone(&self) -> Self {
    Self {
      repr: ContentRepr {
        ptr: self.repr.ptr,
        len: self.repr.len,
        tag: AtomicU32::new(self.repr.tag.load(Ordering::Relaxed)),
      },
      lifetime: PhantomData,
    }
  }
}

impl Deref for ContentSlice<'_> {
  type Target = str;

  fn deref(&self) -> &Self::Target {
    self.as_str()
  }
}
impl AsRef<str> for ContentSlice<'_> {
  fn as_ref(&self) -> &str {
    self.as_str()
  }
}
impl<'a, 'b> PartialEq<ContentSlice<'b>> for ContentSlice<'a> {
  fn eq(&self, other: &ContentSlice<'b>) -> bool {
    self.as_str() == other.as_str()
  }
}
impl Eq for ContentSlice<'_> {}
impl Hash for ContentSlice<'_> {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.as_str().hash(state);
  }
}
impl fmt::Debug for ContentSlice<'_> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.as_str().fmt(f)
  }
}
impl Serialize for ContentSlice<'_> {
  fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    self.as_str().serialize(serializer)
  }
}

/// A string that either owns its UTF-8 bytes or borrows them with an ASCII tag.
pub enum CowContent<'a> {
  Owned(Content),
  Borrowed(ContentSlice<'a>),
}

impl<'a> CowContent<'a> {
  pub fn from_content(value: &'a Content) -> Self {
    Self::Borrowed(value.view())
  }

  /// Borrow the same bytes and ASCII cache as this content.
  pub fn view(&self) -> ContentSlice<'_> {
    match self {
      Self::Owned(value) => value.view(),
      Self::Borrowed(value) => value.share(),
    }
  }

  pub fn as_str(&self) -> &str {
    match self {
      Self::Owned(value) => value.as_str(),
      Self::Borrowed(value) => value.as_str(),
    }
  }

  pub fn is_ascii(&self) -> bool {
    match self {
      Self::Owned(value) => value.is_ascii(),
      Self::Borrowed(value) => value.is_ascii(),
    }
  }

  pub fn cached_is_ascii(&self) -> Option<bool> {
    match self {
      Self::Owned(value) => value.cached_is_ascii(),
      Self::Borrowed(value) => value.cached_is_ascii(),
    }
  }

  pub fn len(&self) -> usize {
    match self {
      Self::Owned(value) => value.len(),
      Self::Borrowed(value) => value.len(),
    }
  }

  pub fn is_empty(&self) -> bool {
    match self {
      Self::Owned(value) => value.is_empty(),
      Self::Borrowed(value) => value.is_empty(),
    }
  }

  pub fn into_owned(self) -> Content {
    match self {
      Self::Owned(value) => value,
      Self::Borrowed(value) => Content::from(value.as_str()),
    }
  }

  pub fn into_cow_str(self) -> Cow<'a, str> {
    match self {
      Self::Owned(value) => Cow::Owned(value.into_string()),
      Self::Borrowed(value) => Cow::Borrowed(value.as_str()),
    }
  }
}

impl<'a> From<Cow<'a, str>> for CowContent<'a> {
  fn from(value: Cow<'a, str>) -> Self {
    match value {
      Cow::Borrowed(value) => Self::Borrowed(ContentSlice::from_str(value)),
      Cow::Owned(value) => Self::Owned(Content::from_boxed(value.into_boxed_str())),
    }
  }
}

impl<'a> From<&'a str> for CowContent<'a> {
  fn from(value: &'a str) -> Self {
    Self::Borrowed(ContentSlice::from_str(value))
  }
}

impl From<String> for CowContent<'_> {
  fn from(value: String) -> Self {
    Self::Owned(Content::from(value))
  }
}

impl From<Content> for CowContent<'_> {
  fn from(value: Content) -> Self {
    Self::Owned(value)
  }
}

impl<'a> From<ContentSlice<'a>> for CowContent<'a> {
  fn from(value: ContentSlice<'a>) -> Self {
    Self::Borrowed(value)
  }
}

impl Clone for CowContent<'_> {
  fn clone(&self) -> Self {
    match self {
      Self::Owned(value) => Self::Owned(value.clone()),
      Self::Borrowed(value) => Self::Borrowed(value.clone()),
    }
  }
}
impl Deref for CowContent<'_> {
  type Target = str;

  fn deref(&self) -> &Self::Target {
    self.as_str()
  }
}
impl AsRef<str> for CowContent<'_> {
  fn as_ref(&self) -> &str {
    self.as_str()
  }
}
impl<'a, 'b> PartialEq<CowContent<'b>> for CowContent<'a> {
  fn eq(&self, other: &CowContent<'b>) -> bool {
    self.as_str() == other.as_str()
  }
}
impl Eq for CowContent<'_> {}
impl Hash for CowContent<'_> {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.as_str().hash(state);
  }
}
impl fmt::Debug for CowContent<'_> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.as_str().fmt(f)
  }
}
impl Serialize for CowContent<'_> {
  fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    self.as_str().serialize(serializer)
  }
}

#[cfg(feature = "allocative")]
impl allocative::Allocative for ContentSlice<'_> {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut allocative::Visitor<'b>) {
    visitor.enter_self_sized::<Self>().exit();
  }
}

#[cfg(feature = "allocative")]
impl allocative::Allocative for CowContent<'_> {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut allocative::Visitor<'b>) {
    let mut visitor = visitor.enter_self_sized::<Self>();
    if let Self::Owned(value) = self {
      value.visit(&mut visitor);
    }
    visitor.exit();
  }
}

// SAFETY: UTF-8 bytes are immutable, all shared targets remain alive for the
// view lifetime, and the only mutation is an AtomicU32 cache update.
unsafe impl Send for Content {}
unsafe impl Sync for Content {}
unsafe impl Send for ContentSlice<'_> {}
unsafe impl Sync for ContentSlice<'_> {}
