use std::borrow::Cow;

use rspack_util::identifier::strip_dos_device_path_prefix;
use smallvec::SmallVec;

use crate::{Error, Result};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum PathFlavor {
  Unix,
  Windows,
}

impl PathFlavor {
  pub(crate) const HOST: Self = if cfg!(windows) {
    Self::Windows
  } else {
    Self::Unix
  };

  #[inline]
  pub(crate) const fn separator(self) -> char {
    match self {
      Self::Unix => '/',
      Self::Windows => '\\',
    }
  }
}

pub(crate) fn validate_root_name(name: &str) -> Result<()> {
  if name.is_empty()
    || name.len() > 254
    || !name
      .bytes()
      .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'@' | b'-'))
  {
    return Err(Error::InvalidRootName(name.to_string()));
  }
  Ok(())
}

pub(crate) fn normalize_abs_path(raw: &str, flavor: PathFlavor) -> Result<Cow<'_, str>> {
  match flavor {
    PathFlavor::Unix => normalize_unix_abs_path(raw),
    PathFlavor::Windows => normalize_windows_abs_path(raw),
  }
}

fn normalize_unix_abs_path(raw: &str) -> Result<Cow<'_, str>> {
  if !raw.starts_with('/') {
    return Err(Error::NotAbsolutePath(raw.to_string()));
  }
  if is_canonical_unix_abs_path(raw) {
    return Ok(Cow::Borrowed(raw));
  }

  let mut parts: SmallVec<[&str; 16]> = SmallVec::new();
  for seg in raw[1..].split('/') {
    match seg {
      "" | "." => {}
      ".." => {
        parts.pop();
      }
      _ => parts.push(seg),
    }
  }
  if parts.is_empty() {
    return Ok(Cow::Owned("/".to_string()));
  }
  let mut out = String::with_capacity(parts.iter().map(|p| 1 + p.len()).sum());
  for part in parts {
    out.push('/');
    out.push_str(part);
  }
  Ok(Cow::Owned(out))
}

pub(crate) fn is_canonical_unix_abs_path(s: &str) -> bool {
  if !s.starts_with('/') {
    return false;
  }
  if s == "/" {
    return true;
  }
  !s.ends_with('/')
    && s[1..]
      .split('/')
      .all(|seg| !seg.is_empty() && seg != "." && seg != "..")
}

#[inline]
fn is_windows_sep_byte(b: u8) -> bool {
  b == b'/' || b == b'\\'
}

#[inline]
fn is_windows_drive_abs_prefix(bytes: &[u8]) -> bool {
  bytes.len() >= 3
    && bytes[0].is_ascii_alphabetic()
    && bytes[1] == b':'
    && is_windows_sep_byte(bytes[2])
}

#[inline]
fn has_windows_drive_prefix(bytes: &[u8]) -> bool {
  bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':'
}

fn parse_windows_unc(s: &str) -> Option<(&str, &str, &str)> {
  let bytes = s.as_bytes();
  if bytes.len() < 2 || !is_windows_sep_byte(bytes[0]) || !is_windows_sep_byte(bytes[1]) {
    return None;
  }
  let rest = &s[2..];
  let server_end = rest.find(['/', '\\'])?;
  let server = &rest[..server_end];
  if server.is_empty() || matches!(server, "." | ".." | "?") {
    return None;
  }
  let after_server = &rest[server_end + 1..];
  let (share, tail) = match after_server.find(['/', '\\']) {
    Some(share_end) => (&after_server[..share_end], &after_server[share_end + 1..]),
    None => (after_server, ""),
  };
  if share.is_empty() || matches!(share, "." | "..") {
    return None;
  }
  Some((server, share, tail))
}

fn normalize_windows_abs_path(raw: &str) -> Result<Cow<'_, str>> {
  let stripped = strip_dos_device_path_prefix(raw);
  let s = stripped.as_ref();
  let bytes = s.as_bytes();

  if is_windows_drive_abs_prefix(bytes) {
    if matches!(stripped, Cow::Borrowed(_)) && is_canonical_windows_drive_path(s) {
      return Ok(Cow::Borrowed(raw));
    }
    let drive = (bytes[0] as char).to_ascii_uppercase();
    let mut parts: SmallVec<[&str; 16]> = SmallVec::new();
    for seg in s[3..].split(['/', '\\']) {
      match seg {
        "" | "." => {}
        ".." => {
          parts.pop();
        }
        _ => parts.push(seg),
      }
    }
    let mut out = String::with_capacity(s.len().max(3));
    out.push(drive);
    out.push_str(":\\");
    for (idx, part) in parts.iter().enumerate() {
      if idx > 0 {
        out.push('\\');
      }
      out.push_str(part);
    }
    return Ok(Cow::Owned(out));
  }

  if let Some((server, share, tail)) = parse_windows_unc(s) {
    if matches!(stripped, Cow::Borrowed(_)) && is_canonical_windows_unc_path(s) {
      return Ok(Cow::Borrowed(raw));
    }
    let mut parts: SmallVec<[&str; 16]> = SmallVec::new();
    for seg in tail.split(['/', '\\']) {
      match seg {
        "" | "." => {}
        ".." => {
          parts.pop();
        }
        _ => parts.push(seg),
      }
    }
    let mut out = String::with_capacity(s.len() + 2);
    out.push_str("\\\\");
    out.push_str(server);
    out.push('\\');
    out.push_str(share);
    out.push('\\');
    for (idx, part) in parts.iter().enumerate() {
      if idx > 0 {
        out.push('\\');
      }
      out.push_str(part);
    }
    return Ok(Cow::Owned(out));
  }

  Err(Error::NotAbsolutePath(raw.to_string()))
}

fn is_canonical_windows_drive_path(s: &str) -> bool {
  let bytes = s.as_bytes();
  if !(bytes.len() >= 3 && bytes[0].is_ascii_uppercase() && bytes[1] == b':' && bytes[2] == b'\\') {
    return false;
  }
  if bytes.len() == 3 {
    return true;
  }
  !s.ends_with('\\')
    && s[3..]
      .split('\\')
      .all(|seg| !seg.is_empty() && seg != "." && seg != ".." && !seg.contains('/'))
}

fn is_canonical_windows_unc_path(s: &str) -> bool {
  if !s.starts_with("\\\\") {
    return false;
  }
  let rest = &s[2..];
  let Some(server_end) = rest.find('\\') else {
    return false;
  };
  let server = &rest[..server_end];
  if server.is_empty() || server.contains('/') || matches!(server, "." | ".." | "?") {
    return false;
  }
  let after_server = &rest[server_end + 1..];
  let Some(share_end) = after_server.find('\\') else {
    return false;
  };
  let share = &after_server[..share_end];
  if share.is_empty() || share.contains('/') || matches!(share, "." | "..") {
    return false;
  }
  let tail = &after_server[share_end + 1..];
  if tail.is_empty() {
    return true;
  }
  !tail.ends_with('\\')
    && tail
      .split('\\')
      .all(|seg| !seg.is_empty() && seg != "." && seg != ".." && !seg.contains('/'))
}

pub(crate) fn is_canonical_windows_abs_path(s: &str) -> bool {
  is_canonical_windows_drive_path(s) || is_canonical_windows_unc_path(s)
}

pub(crate) fn is_canonical_rel_path(s: &str) -> bool {
  if s.is_empty() {
    return true;
  }
  if s.starts_with('/') || s.ends_with('/') || s.contains('\\') {
    return false;
  }
  let mut seen_normal = false;
  for seg in s.split('/') {
    if seg.is_empty() || seg == "." {
      return false;
    }
    if seg == ".." {
      if seen_normal {
        return false;
      }
    } else {
      seen_normal = true;
    }
  }
  true
}

fn format_rel_segments(leading_parents: usize, parts: &[&str]) -> String {
  if leading_parents == 0 && parts.is_empty() {
    return String::new();
  }
  let count = leading_parents + parts.len();
  let len =
    leading_parents * 2 + parts.iter().map(|p| p.len()).sum::<usize>() + count.saturating_sub(1);
  let mut out = String::with_capacity(len);
  for i in 0..leading_parents {
    if i > 0 {
      out.push('/');
    }
    out.push_str("..");
  }
  for part in parts {
    if !out.is_empty() {
      out.push('/');
    }
    out.push_str(part);
  }
  out
}

pub(crate) fn normalize_rel_path(raw: &str) -> Result<Cow<'_, str>> {
  if raw.is_empty() {
    return Ok(Cow::Borrowed(""));
  }
  let bytes = raw.as_bytes();
  if is_windows_sep_byte(bytes[0]) || has_windows_drive_prefix(bytes) {
    return Err(Error::NotRelativePath(raw.to_string()));
  }
  if is_canonical_rel_path(raw) {
    return Ok(Cow::Borrowed(raw));
  }

  let mut leading_parents = 0usize;
  let mut parts: SmallVec<[&str; 16]> = SmallVec::new();
  for seg in raw.split(['/', '\\']) {
    match seg {
      "" | "." => {}
      ".." => {
        if parts.pop().is_none() {
          leading_parents += 1;
        }
      }
      _ => parts.push(seg),
    }
  }
  Ok(Cow::Owned(format_rel_segments(leading_parents, &parts)))
}

pub(crate) fn normalize_rel_join(base: &str, sub: &str) -> Result<String> {
  if sub.is_empty() {
    return Ok(base.to_string());
  }
  let sub_bytes = sub.as_bytes();
  if is_windows_sep_byte(sub_bytes[0]) || has_windows_drive_prefix(sub_bytes) {
    return Err(Error::NotRelativePath(sub.to_string()));
  }
  if base.is_empty() {
    return normalize_rel_path(sub).map(Cow::into_owned);
  }

  let sub_has_parent = sub == ".." || sub.starts_with("../");
  if is_canonical_rel_path(sub) && (!sub_has_parent || base.split('/').all(|seg| seg == "..")) {
    let mut out = String::with_capacity(base.len() + 1 + sub.len());
    out.push_str(base);
    out.push('/');
    out.push_str(sub);
    return Ok(out);
  }

  let mut leading_parents = 0usize;
  let mut parts: SmallVec<[&str; 16]> = SmallVec::new();
  for seg in base.split('/').chain(sub.split(['/', '\\'])) {
    match seg {
      "" | "." => {}
      ".." => {
        if parts.pop().is_none() {
          leading_parents += 1;
        }
      }
      _ => parts.push(seg),
    }
  }
  Ok(format_rel_segments(leading_parents, &parts))
}

pub(crate) fn abs_relative_to(target: &str, base: &str, flavor: PathFlavor) -> Option<String> {
  if target == base {
    return Some(String::new());
  }
  let t_root = abs_root_len(target, flavor);
  let b_root = abs_root_len(base, flavor);
  let root_matches = match flavor {
    PathFlavor::Unix => target[..t_root] == base[..b_root],
    PathFlavor::Windows => target[..t_root].eq_ignore_ascii_case(&base[..b_root]),
  };
  if !root_matches {
    return None;
  }

  let sep = flavor.separator();
  let t_tail = &target[t_root..];
  let b_tail = &base[b_root..];
  let t_segs: SmallVec<[&str; 16]> = if t_tail.is_empty() {
    SmallVec::new()
  } else {
    t_tail.split(sep).collect()
  };
  let b_segs: SmallVec<[&str; 16]> = if b_tail.is_empty() {
    SmallVec::new()
  } else {
    b_tail.split(sep).collect()
  };
  let common = t_segs
    .iter()
    .zip(b_segs.iter())
    .take_while(|(a, b)| a == b)
    .count();
  let up_count = b_segs.len() - common;
  Some(format_rel_segments(up_count, &t_segs[common..]))
}

pub(crate) fn normalize_posix_rel_path(raw: &str) -> Result<Cow<'_, str>> {
  if raw.is_empty() {
    return Ok(Cow::Borrowed(""));
  }
  let bytes = raw.as_bytes();
  if is_windows_sep_byte(bytes[0]) || has_windows_drive_prefix(bytes) {
    return Err(Error::PathEscapesRoot(raw.to_string()));
  }
  if is_canonical_posix_rel_path(raw) {
    return Ok(Cow::Borrowed(raw));
  }

  let mut parts: SmallVec<[&str; 16]> = SmallVec::new();
  for seg in raw.split(['/', '\\']) {
    match seg {
      "" | "." => {}
      ".." => {
        if parts.pop().is_none() {
          return Err(Error::PathEscapesRoot(raw.to_string()));
        }
      }
      _ => parts.push(seg),
    }
  }
  Ok(Cow::Owned(parts.join("/")))
}

pub(crate) fn is_canonical_posix_rel_path(s: &str) -> bool {
  !s.is_empty()
    && !s.starts_with('/')
    && !s.ends_with('/')
    && s
      .split('/')
      .all(|seg| !seg.is_empty() && seg != "." && seg != ".." && !seg.contains('\\'))
}

pub(crate) fn abs_root_len(path: &str, flavor: PathFlavor) -> usize {
  match flavor {
    PathFlavor::Unix => 1,
    PathFlavor::Windows => {
      let bytes = path.as_bytes();
      if has_windows_drive_prefix(bytes) {
        3
      } else {
        let rest = &path[2..];
        let server_end = rest.find('\\').expect("canonical UNC path has server");
        let after_server = &rest[server_end + 1..];
        let share_end = after_server
          .find('\\')
          .expect("canonical UNC path has share");
        2 + server_end + 1 + share_end + 1
      }
    }
  }
}

pub(crate) fn abs_parent(path: &str, flavor: PathFlavor) -> Option<&str> {
  let root_len = abs_root_len(path, flavor);
  if path.len() <= root_len {
    return None;
  }
  let idx = path.rfind(flavor.separator())?;
  if idx < root_len {
    Some(&path[..root_len])
  } else {
    Some(&path[..idx])
  }
}

pub(crate) fn abs_strip_prefix<'a>(
  path: &'a str,
  base: &str,
  flavor: PathFlavor,
) -> Option<&'a str> {
  if path == base {
    return Some("");
  }
  let base_root_len = abs_root_len(base, flavor);
  if base.len() == base_root_len {
    return path.strip_prefix(base);
  }
  let tail = path.strip_prefix(base)?;
  tail.strip_prefix(flavor.separator())
}

pub(crate) fn abs_join<'a>(
  base: &'a str,
  sub: &'a str,
  flavor: PathFlavor,
) -> Result<Cow<'a, str>> {
  if sub.is_empty() {
    return Ok(Cow::Borrowed(base));
  }
  match flavor {
    PathFlavor::Unix => {
      if sub.starts_with('/') {
        return normalize_unix_abs_path(sub);
      }
      if !sub.ends_with('/')
        && sub
          .split('/')
          .all(|seg| !seg.is_empty() && seg != "." && seg != "..")
      {
        let needs_sep = !base.ends_with('/');
        let mut out = String::with_capacity(base.len() + usize::from(needs_sep) + sub.len());
        out.push_str(base);
        if needs_sep {
          out.push('/');
        }
        out.push_str(sub);
        return Ok(Cow::Owned(out));
      }

      let mut parts: SmallVec<[&str; 16]> = SmallVec::new();
      if base.len() > 1 {
        parts.extend(base[1..].split('/'));
      }
      for seg in sub.split('/') {
        match seg {
          "" | "." => {}
          ".." => {
            parts.pop();
          }
          _ => parts.push(seg),
        }
      }
      if parts.is_empty() {
        return Ok(Cow::Owned("/".to_string()));
      }
      let mut out = String::with_capacity(parts.iter().map(|p| 1 + p.len()).sum());
      for part in parts {
        out.push('/');
        out.push_str(part);
      }
      Ok(Cow::Owned(out))
    }
    PathFlavor::Windows => {
      let stripped = strip_dos_device_path_prefix(sub);
      let s = stripped.as_ref();
      let bytes = s.as_bytes();

      if is_windows_drive_abs_prefix(bytes)
        || (bytes.len() >= 2 && is_windows_sep_byte(bytes[0]) && is_windows_sep_byte(bytes[1]))
      {
        return normalize_windows_abs_path(sub);
      }
      if has_windows_drive_prefix(bytes) {
        return Err(Error::NotAbsolutePath(sub.to_string()));
      }

      let root_len = abs_root_len(base, PathFlavor::Windows);
      let mut parts: SmallVec<[&str; 16]> = SmallVec::new();
      let rel_sub = if is_windows_sep_byte(bytes[0]) {
        &s[1..]
      } else {
        if base.len() > root_len {
          parts.extend(base[root_len..].split('\\'));
        }
        sub
      };
      for seg in rel_sub.split(['/', '\\']) {
        match seg {
          "" | "." => {}
          ".." => {
            parts.pop();
          }
          _ => parts.push(seg),
        }
      }

      let tail_len = if parts.is_empty() {
        0
      } else {
        parts.iter().map(|p| p.len()).sum::<usize>() + (parts.len() - 1)
      };
      let mut out = String::with_capacity(root_len + tail_len);
      out.push_str(&base[..root_len]);
      for (idx, part) in parts.iter().enumerate() {
        if idx > 0 {
          out.push('\\');
        }
        out.push_str(part);
      }
      Ok(Cow::Owned(out))
    }
  }
}
