use std::borrow::Cow;

use camino::{Utf8Path, Utf8PathBuf};
use serde_json::{Map, Value};

use crate::{ResolveError, path::PathUtil};

/// Parsed Node.js package map and its package-location index.
///
/// The map stays in the JSON objects it was parsed into, which the resolver already uses for
/// other files, rather than in dedicated collections that would add to the size of the binary.
pub(crate) struct PackageMap {
  path: Utf8PathBuf,
  /// The `packages` object: package ID -> `{ "url", "dependencies" }`, where `dependencies` maps
  /// the package name of a bare specifier to a package ID. Each `url` is replaced by the
  /// location it resolves to.
  packages: Map<String, Value>,
  /// Package location -> the package ID that owns it, or `false` when several IDs share it.
  owners: Map<String, Value>,
}

impl PackageMap {
  /// Parses a package map, see
  /// [Configuration file format](https://nodejs.org/api/packages.html#configuration-file-format).
  ///
  /// The error is the reason why the map is invalid.
  pub(crate) fn parse(path: &Utf8Path, json: &str) -> Result<Self, String> {
    let invalid_package =
      |package_id: &str, problem: &str| format!("package {package_id:?} {problem}");
    let mut data = serde_json::from_str::<Value>(json).map_err(|error| error.to_string())?;
    let Some(Value::Object(mut packages)) = data.get_mut("packages").map(Value::take) else {
      return Err("expected an object with a \"packages\" object".to_string());
    };
    let base = path
      .parent()
      .ok_or_else(|| "an invalid configuration file path".to_string())?;
    let mut owners = Map::new();

    for (package_id, entry) in &mut packages {
      let mut location = None;
      let mut has_valid_dependencies = true;
      for (key, value) in entry.as_object_mut().into_iter().flatten() {
        match (key.as_str(), value) {
          ("url", Value::String(url)) => {
            let resolved = resolve_url(base, url)
              .map_err(|reason| invalid_package(package_id, &format!("has {reason}")))?
              .into_string();
            *url = resolved.clone();
            location = Some(resolved);
          }
          ("dependencies", Value::Object(dependencies)) => {
            has_valid_dependencies = dependencies.values().all(Value::is_string);
          }
          ("dependencies", _) => has_valid_dependencies = false,
          _ => {}
        }
      }
      let Some(location) = location else {
        return Err(invalid_package(
          package_id,
          "must have a string \"url\" field",
        ));
      };
      if !has_valid_dependencies {
        return Err(invalid_package(
          package_id,
          "must have a \"dependencies\" object of package IDs",
        ));
      }

      // Several package IDs can share a location, which then has no single owner.
      let owner = owners.entry(location).or_insert(Value::Null);
      *owner = if owner.is_null() {
        Value::String(package_id.clone())
      } else {
        Value::Bool(false)
      };
    }

    Ok(Self {
      path: path.to_path_buf(),
      packages,
      owners,
    })
  }

  /// Finds the location of the package that `package_name` refers to in the package that contains
  /// the directory `importer`.
  ///
  /// Implements package-map dispatch from step 6 of Node's CommonJS resolution pseudocode and
  /// steps 2 to 6 of `LOAD_PACKAGE_MAP(X, PARENT_PACKAGE_ID, PACKAGE_MAP)`. Node permits the
  /// importing package ID to be propagated by the caller. The resolver returns paths rather than
  /// package IDs, so the importing path is used to find it.
  ///
  /// Returns `None` when the importing package does not depend on `package_name`.
  ///
  /// See <https://nodejs.org/api/modules.html#all-together>.
  pub(crate) fn target(
    &self,
    importer: &Utf8Path,
    specifier: &str,
    package_name: &str,
  ) -> Result<Option<&Utf8Path>, ResolveError> {
    // 6.a. Let PARENT_PACKAGE_ID be FIND_PACKAGE_ID(dirname(Y), PACKAGE_MAP)
    //
    // The nearest ancestor that is a package location owns the importer. Locations are compared
    // in their normalized form.
    let importer = importer.normalize();
    let owner = importer
      .ancestors()
      .find_map(|path| self.owners.get(path.as_str()));
    let Some(Value::String(parent_package_id)) = owner else {
      let specifier = specifier.to_string();
      let parent_path = importer.into_std_path_buf();
      let package_map_path = self.path.as_std_path().to_path_buf();
      return Err(if owner.is_some() {
        ResolveError::PackageMapAmbiguousResolution {
          specifier,
          parent_path,
          package_map_path,
        }
      } else {
        ResolveError::PackageMapExternalFile {
          specifier,
          parent_path,
          package_map_path,
        }
      });
    };

    // 2. Find the package map entry for key PARENT_PACKAGE_ID.
    // 3. Look up NAME in the entry's "dependencies" map.
    // 4. If NAME is not found, THROW "not found".
    let Some(dependency_id) = self
      .packages
      .get(parent_package_id)
      .and_then(|package| package.get("dependencies"))
      .and_then(|dependencies| dependencies.get(package_name))
      .and_then(Value::as_str)
    else {
      return Ok(None);
    };
    // 5. Let TARGET be PACKAGE_MAP.packages[dependencies[name]].
    // 6. Let PACKAGE_PATH be the resolved path of TARGET.
    let Some(package_path) = self
      .packages
      .get(dependency_id)
      .and_then(|package| package.get("url"))
      .and_then(Value::as_str)
    else {
      return Err(ResolveError::PackageMapInvalid {
        package_map_path: self.path.as_std_path().to_path_buf(),
        reason: format!("package ID {dependency_id:?} is a dependency but is not defined"),
      });
    };
    tracing::debug!(
      parent_package_id,
      dependency_id,
      package_path,
      "resolve_package_map"
    );
    Ok(Some(Utf8Path::new(package_path)))
  }
}

/// Resolves an entry URL against the directory of the package map.
///
/// Node.js parses these URLs with the WHATWG URL parser and only supports the `file:` protocol.
/// Relative URLs and the forms of `file:` URLs are supported here, with percent-encoding, a query
/// and a fragment.
fn resolve_url(base: &Utf8Path, value: &str) -> Result<Utf8PathBuf, String> {
  if value.is_empty() {
    return Err("an empty \"url\" field".to_string());
  }
  // The query and the fragment do not take part in the file path.
  let value = value.split(['?', '#']).next().unwrap_or_default().trim();
  let invalid = || format!("an invalid file URL {value:?}");

  let path = match split_url_scheme(value) {
    Some((scheme, path)) if scheme.eq_ignore_ascii_case("file") => path,
    Some(_) => {
      return Err(format!(
        "an unsupported URL scheme in {value:?}; only file URLs and relative URLs are supported"
      ));
    }
    None => value,
  };
  // `//host/path` has an absolute path.
  let (host, path) = match path.strip_prefix("//") {
    Some(rest) => {
      let host = rest.split('/').next().unwrap_or_default();
      if matches!(host.as_bytes(), [drive, b':'] if drive.is_ascii_alphabetic()) {
        // `file://C:/path` holds a drive letter where the host is expected.
        ("", Cow::Owned(format!("/{rest}")))
      } else if host.len() == rest.len() {
        (host, Cow::Borrowed("/"))
      } else {
        (host, Cow::Borrowed(&rest[host.len()..]))
      }
    }
    None => ("", Cow::Borrowed(path)),
  };
  // Like `fileURLToPath`, the only host of a path is the local one, except for the server of a
  // UNC path on Windows.
  let is_local = host.is_empty() || host.eq_ignore_ascii_case("localhost");
  if !is_local && (cfg!(not(windows)) || host.contains(['@', ':', '%', '[', ' '])) {
    return Err(invalid());
  }
  let path = percent_decode(&path).ok_or_else(invalid)?;
  if path.is_empty() {
    return Ok(base.to_path_buf());
  }

  #[cfg(windows)]
  if !is_local {
    use cow_utils::CowUtils;

    let path = path.cow_replace('/', "\\");
    return Ok(Utf8PathBuf::from(format!(r"\\{host}{path}")).normalize());
  }
  #[cfg(windows)]
  if path.starts_with('/') {
    return windows_root_relative_path(base, &path);
  }
  Ok(base.normalize_with(path.as_ref()).normalize())
}

/// Resolves a root-relative URL (`/path`) against the drive or the server of the package map.
#[cfg(windows)]
fn windows_root_relative_path(base: &Utf8Path, relative: &str) -> Result<Utf8PathBuf, String> {
  use camino::{Utf8Component, Utf8Prefix};

  let invalid_base = || "an invalid configuration file path".to_string();
  let Some(Utf8Component::Prefix(prefix)) = base.components().next() else {
    return Err(invalid_base());
  };
  match prefix.kind() {
    Utf8Prefix::Disk(drive) | Utf8Prefix::VerbatimDisk(drive) => {
      // `/C:/path` selects another drive.
      let (drive, suffix) = match relative.as_bytes() {
        [b'/', drive, b':', b'/', ..] if drive.is_ascii_alphabetic() => (*drive, &relative[3..]),
        _ => (drive, relative),
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

/// Decodes the percent-encoded bytes of a URL path.
///
/// Returns `None` for an invalid escape, for bytes that are not UTF-8 and, like `fileURLToPath`,
/// for an encoded path separator.
fn percent_decode(value: &str) -> Option<Cow<'_, str>> {
  if !value.contains('%') {
    return Some(Cow::Borrowed(value));
  }
  let mut decoded = Vec::with_capacity(value.len());
  let mut bytes = value.bytes();
  while let Some(byte) = bytes.next() {
    if byte != b'%' {
      decoded.push(byte);
      continue;
    }
    let high = char::from(bytes.next()?).to_digit(16)?;
    let low = char::from(bytes.next()?).to_digit(16)?;
    let byte = (high << 4 | low) as u8;
    if byte == b'/' || (cfg!(windows) && byte == b'\\') {
      return None;
    }
    decoded.push(byte);
  }
  String::from_utf8(decoded).ok().map(Cow::Owned)
}
