use std::{borrow::Cow, path::Path};

use camino::{Utf8Component, Utf8Path, Utf8PathBuf};
use rustc_hash::FxHashMap;
use serde::Deserialize;

use crate::{ResolveError, path::PathUtil};

/// Error returned by the path-based fallback in Node's package-map resolution algorithm.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum FindPackageIdError {
  /// Multiple package IDs share the location that contains the importer.
  AmbiguousResolution,
  /// No mapped package contains the importer.
  ExternalFile,
}

/// The package map as written on disk.
///
/// See [Configuration file format](https://nodejs.org/api/packages.html#configuration-file-format).
#[derive(Deserialize)]
struct PackageMapData {
  packages: FxHashMap<String, PackageMapEntryData>,
}

#[derive(Deserialize)]
struct PackageMapEntryData {
  url: String,
  #[serde(default)]
  dependencies: FxHashMap<String, String>,
}

/// A package of a [PackageMap].
pub(crate) struct PackageMapEntry {
  /// The location decoded from the entry's `url`.
  path: Utf8PathBuf,
  /// Bare specifier package name -> package ID.
  dependencies: FxHashMap<String, String>,
}

impl PackageMapEntry {
  pub(crate) fn path(&self) -> &Utf8Path {
    &self.path
  }

  pub(crate) fn dependency(&self, name: &str) -> Option<&str> {
    self.dependencies.get(name).map(String::as_str)
  }
}

enum PackageOwner {
  Package(String),
  Ambiguous,
}

/// Parsed Node.js package map and its package-location index.
pub(crate) struct PackageMap {
  path: Utf8PathBuf,
  packages: FxHashMap<String, PackageMapEntry>,
  /// Package location -> the package ID that owns it.
  owners: FxHashMap<Utf8PathBuf, PackageOwner>,
}

impl PackageMap {
  pub(crate) fn parse(path: &Utf8Path, json: &str) -> Result<Self, ResolveError> {
    let data = serde_json::from_str::<PackageMapData>(json).map_err(|error| {
      if error.is_data() {
        Self::invalid(path, error.to_string())
      } else {
        ResolveError::from_serde_json_error(
          path.as_std_path().to_path_buf(),
          &error,
          Some(json.to_string()),
        )
      }
    })?;

    let base = path
      .parent()
      .ok_or_else(|| Self::invalid(path, "an invalid configuration file path".to_string()))?;
    let mut packages = FxHashMap::with_capacity_and_hasher(data.packages.len(), Default::default());
    let mut owners = FxHashMap::with_capacity_and_hasher(data.packages.len(), Default::default());

    for (package_id, entry) in data.packages {
      if entry.url.is_empty() {
        return Err(Self::invalid(
          path,
          format!("package {package_id:?} has an empty \"url\" field"),
        ));
      }
      let package_path = resolve_url(base, &entry.url)
        .map_err(|reason| Self::invalid(path, format!("package {package_id:?} has {reason}")))?;

      owners
        .entry(package_path.clone())
        .and_modify(|owner| *owner = PackageOwner::Ambiguous)
        .or_insert_with(|| PackageOwner::Package(package_id.clone()));
      packages.insert(
        package_id,
        PackageMapEntry {
          path: package_path,
          dependencies: entry.dependencies,
        },
      );
    }

    Ok(Self {
      path: path.to_path_buf(),
      packages,
      owners,
    })
  }

  pub(crate) fn path(&self) -> &Path {
    self.path.as_std_path()
  }

  pub(crate) fn package(&self, package_id: &str) -> Option<&PackageMapEntry> {
    self.packages.get(package_id)
  }

  /// Implements the path-based fallback for determining which package owns an importer.
  ///
  /// The nearest ancestor that is a package location owns `path`. If multiple package IDs share
  /// that location, this returns [FindPackageIdError::AmbiguousResolution]. If no mapped package
  /// contains `path`, it returns [FindPackageIdError::ExternalFile].
  ///
  /// This corresponds to `FIND_PACKAGE_ID(PATH, PACKAGE_MAP)` in Node's
  /// [CommonJS resolution pseudocode](https://nodejs.org/api/modules.html#all-together).
  pub(crate) fn find_package_id(&self, path: &Utf8Path) -> Result<&str, FindPackageIdError> {
    let normalized_path;
    let path = if path
      .components()
      .any(|component| component == Utf8Component::ParentDir)
    {
      normalized_path = path.normalize();
      normalized_path.as_path()
    } else {
      path
    };
    match path.ancestors().find_map(|path| self.owners.get(path)) {
      Some(PackageOwner::Package(package_id)) => Ok(package_id),
      Some(PackageOwner::Ambiguous) => Err(FindPackageIdError::AmbiguousResolution),
      None => Err(FindPackageIdError::ExternalFile),
    }
  }

  pub(crate) fn invalid(path: &Utf8Path, reason: String) -> ResolveError {
    ResolveError::PackageMapInvalid {
      package_map_path: path.as_std_path().to_path_buf(),
      reason,
    }
  }
}

/// Resolves an entry URL against the directory of the package map using WHATWG URL semantics.
///
/// Only `file:` URLs and relative URLs are supported.
fn resolve_url(base: &Utf8Path, value: &str) -> Result<Utf8PathBuf, String> {
  // WHATWG URL parsing trims leading and trailing C0 controls and spaces, and removes ASCII
  // tabs and newlines anywhere in the input. Backslashes are path separators for `file:`
  // URLs, including relative URLs resolved against a `file:` base.
  let value = value.trim_matches(|character: char| character <= '\u{20}');
  let value = if value.contains(['\\', '\t', '\n', '\r']) {
    Cow::Owned(
      value
        .chars()
        .filter_map(|character| match character {
          '\t' | '\n' | '\r' => None,
          '\\' => Some('/'),
          character => Some(character),
        })
        .collect::<String>(),
    )
  } else {
    Cow::Borrowed(value)
  };
  // The query and the fragment do not take part in the file path.
  let value = value
    .split_once(['?', '#'])
    .map_or(value.as_ref(), |(path, _)| path);
  let invalid = || format!("an invalid file URL {value:?}");

  let scheme = split_url_scheme(value);
  if scheme.is_some_and(|(scheme, _)| !scheme.eq_ignore_ascii_case("file")) {
    return Err(format!(
      "an unsupported URL scheme in {value:?}; only file URLs and relative URLs are supported"
    ));
  }
  if has_invalid_percent_encoding(value) {
    return Err(invalid());
  }

  let relative = match scheme {
    // Protocol-relative URL, such as `//localhost/path`.
    None if value.starts_with("//") => {
      return file_url_to_path(&value[2..]).ok_or_else(invalid);
    }
    None => value,
    // `file://host/path` and `file:///path`.
    Some((_, rest)) if rest.starts_with("//") => {
      return file_url_to_path(&rest[2..]).ok_or_else(invalid);
    }
    // `file:C:/path`.
    Some((_, rest)) if starts_with_drive_letter(rest) && rest.as_bytes().get(2) == Some(&b'/') => {
      return file_url_to_path(rest).ok_or_else(invalid);
    }
    // `file:path` and `file:/path` are resolved from the package map like relative URLs.
    Some((_, rest)) => rest,
  };

  if has_encoded_separators(relative) {
    return Err(invalid());
  }
  if relative.is_empty() {
    return Ok(base.to_path_buf());
  }
  let decoded = percent_decode(relative).ok_or_else(invalid)?;
  #[cfg(windows)]
  if decoded.starts_with('/') {
    return windows_root_relative_path(base, &decoded);
  }
  Ok(base.normalize_with(decoded.as_ref()).normalize())
}

/// Converts the part of a `file:` URL that follows `file://` to a path.
///
/// Follows Node's `fileURLToPath`: a host is only accepted on Windows, where it creates a UNC path.
fn file_url_to_path(after_scheme: &str) -> Option<Utf8PathBuf> {
  let authority = after_scheme.split('/').next().unwrap_or_default();
  // `file://C:/path` holds a drive letter where the host is expected.
  let (host, pathname) = if is_drive_authority(authority) {
    ("", after_scheme)
  } else {
    (
      authority,
      after_scheme[authority.len()..]
        .strip_prefix('/')
        .unwrap_or_default(),
    )
  };

  let host = percent_decode(host)?;
  if has_invalid_host(&host) {
    return None;
  }
  // The WHATWG URL parser normalizes the "localhost" host of a `file:` URL to the empty host.
  let host = if host.eq_ignore_ascii_case("localhost") {
    ""
  } else {
    host.as_ref()
  };

  if has_encoded_separators(pathname) {
    return None;
  }
  let pathname = percent_decode(pathname)?;

  #[cfg(not(windows))]
  {
    if !host.is_empty() {
      return None;
    }
    Some(Utf8PathBuf::from(format!("/{pathname}")).normalize())
  }
  #[cfg(windows)]
  {
    let pathname = pathname.replace('/', "\\");
    if !host.is_empty() {
      return Some(Utf8PathBuf::from(format!(r"\\{host}\{pathname}")).normalize());
    }
    starts_with_drive_letter(&pathname).then(|| Utf8PathBuf::from(pathname).normalize())
  }
}

/// Resolves a root-relative URL (`/path`) against the drive or the host of the package map.
#[cfg(windows)]
fn windows_root_relative_path(base: &Utf8Path, relative: &str) -> Result<Utf8PathBuf, String> {
  use camino::Utf8Prefix;

  let invalid_base = || "an invalid configuration file path".to_string();
  let Some(Utf8Component::Prefix(prefix)) = base.components().next() else {
    return Err(invalid_base());
  };
  match prefix.kind() {
    Utf8Prefix::Disk(drive) | Utf8Prefix::VerbatimDisk(drive) => {
      // `/C:/path` selects another drive.
      let (drive, suffix) =
        if starts_with_drive_letter(&relative[1..]) && relative.as_bytes().get(3) == Some(&b'/') {
          (relative.as_bytes()[1], &relative[3..])
        } else {
          (drive, relative)
        };
      let mut path = Utf8PathBuf::from(format!("{}:", char::from(drive)));
      path.push(suffix);
      Ok(path.normalize())
    }
    Utf8Prefix::UNC(server, _) | Utf8Prefix::VerbatimUNC(server, _) => {
      Ok(Utf8PathBuf::from(format!(r"\\{server}\{}", relative.trim_start_matches('/'))).normalize())
    }
    Utf8Prefix::DeviceNS(_) | Utf8Prefix::Verbatim(_) => Err(invalid_base()),
  }
}

fn split_url_scheme(value: &str) -> Option<(&str, &str)> {
  if !value.as_bytes().first()?.is_ascii_alphabetic() {
    return None;
  }
  for (index, byte) in value.bytes().enumerate().skip(1) {
    if byte == b':' {
      return Some((&value[..index], &value[index + 1..]));
    }
    if !byte.is_ascii_alphanumeric() && !matches!(byte, b'+' | b'-' | b'.') {
      return None;
    }
  }
  None
}

/// Whether `value` starts with a Windows drive letter and its colon, e.g. `C:`.
fn starts_with_drive_letter(value: &str) -> bool {
  matches!(value.as_bytes(), [drive, b':', ..] if drive.is_ascii_alphabetic())
}

fn is_drive_authority(authority: &str) -> bool {
  authority.len() == 2 && starts_with_drive_letter(authority)
}

fn has_invalid_host(host: &str) -> bool {
  if host.is_empty() {
    return false;
  }
  if host.contains(['@', '/', '\\']) || host.chars().any(|character| character <= '\u{20}') {
    return true;
  }
  if let Some(ipv6) = host.strip_prefix('[') {
    return ipv6
      .strip_suffix(']')
      .is_none_or(|address| address.parse::<std::net::Ipv6Addr>().is_err());
  }
  host.contains(':')
}

/// Whether `value` has a `%` that does not start a percent-encoded byte.
fn has_invalid_percent_encoding(value: &str) -> bool {
  let bytes = value.as_bytes();
  let mut index = 0;
  while index < bytes.len() {
    if bytes[index] == b'%' {
      if !(bytes.get(index + 1).is_some_and(u8::is_ascii_hexdigit)
        && bytes.get(index + 2).is_some_and(u8::is_ascii_hexdigit))
      {
        return true;
      }
      index += 3;
    } else {
      index += 1;
    }
  }
  false
}

/// Whether `pathname` has a percent-encoded path separator, which `fileURLToPath` rejects:
/// `%2F` (`/`) and, on Windows, `%5C` (`\`).
fn has_encoded_separators(pathname: &str) -> bool {
  pathname.as_bytes().windows(3).any(|window| {
    window[0] == b'%'
      && ((window[1] == b'2' && window[2].eq_ignore_ascii_case(&b'f'))
        || (cfg!(windows) && window[1] == b'5' && window[2].eq_ignore_ascii_case(&b'c')))
  })
}

/// Decodes the percent-encoded bytes of `value`, which [has_invalid_percent_encoding] accepted.
///
/// Returns `None` when the decoded bytes are not UTF-8.
fn percent_decode(value: &str) -> Option<Cow<'_, str>> {
  if !value.contains('%') {
    return Some(Cow::Borrowed(value));
  }
  let hex = |byte: u8| char::from(byte).to_digit(16).map(|digit| digit as u8);
  let bytes = value.as_bytes();
  let mut decoded = Vec::with_capacity(bytes.len());
  let mut index = 0;
  while index < bytes.len() {
    if bytes[index] == b'%' {
      let high = hex(*bytes.get(index + 1)?)?;
      let low = hex(*bytes.get(index + 2)?)?;
      decoded.push(high << 4 | low);
      index += 3;
    } else {
      decoded.push(bytes[index]);
      index += 1;
    }
  }
  String::from_utf8(decoded).ok().map(Cow::Owned)
}
