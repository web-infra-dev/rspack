// Adapted from webpack/lib/sharing/utils.js (MIT licensed).
// https://github.com/webpack/webpack/blob/v5.111.1/lib/sharing/utils.js

use std::{borrow::Cow, sync::LazyLock};

use cow_utils::CowUtils;
use regex::Regex;
use url::Url;

static VERSION_PATTERN: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"^(?:[0-9^=v<>~]|[*xX]$)").expect("invalid version regex"));
static GITHUB_SHORTHAND: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new(r"^[^/@:.\s][^/@:\s]*/[^@:\s]*[^/@:\s]#\S+").expect("invalid GitHub shorthand regex")
});
static SHORT_GIT_URL: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new(r"^(?:github|gitlab|bitbucket|gist):/?[^/.]+/?").expect("invalid short git URL regex")
});
static CUSTOM_PROTOCOL: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new(r"^(?:(?:git\+)?(?:ssh|https?|file)|git)://").expect("invalid git protocol regex")
});
static HOSTNAME_WITH_COLON: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new(r"([^/@#:.]+(?:\.[^/@#:.]+)+|localhost):([^#/0-9]+)")
    .expect("invalid git hostname regex")
});
static NO_PROTOCOL: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new(r"^[^/@#:.]+(?:\.[^/@#:.]+)+").expect("invalid protocol-less URL regex")
});
static HOSTNAME: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new(r"^(?:[^/.]+(?:\.[^/.]+)+|localhost)$").expect("invalid hostname regex")
});

pub(super) fn normalize_version(version: &str) -> String {
  let version = version.trim();
  if VERSION_PATTERN.is_match(version) {
    return version.to_owned();
  }

  let version = version.cow_to_lowercase();
  let git_url = if GITHUB_SHORTHAND.is_match(&version) {
    format!("github:{version}")
  } else if SHORT_GIT_URL.is_match(&version) || CUSTOM_PROTOCOL.is_match(&version) {
    version.to_string()
  } else {
    format!("git+ssh://{version}")
  };
  let git_url = HOSTNAME_WITH_COLON.replace(&git_url, "$1/$2");
  let Ok(parsed) = Url::parse(&git_url) else {
    return String::new();
  };
  if !matches!(
    parsed.scheme(),
    "ssh"
      | "http"
      | "https"
      | "file"
      | "git"
      | "git+ssh"
      | "git+http"
      | "git+https"
      | "git+file"
      | "github"
      | "gitlab"
      | "bitbucket"
      | "gist"
  ) || parsed.path().is_empty()
    || decode_url_component(parsed.path()).is_none()
    || (NO_PROTOCOL.is_match(&version)
      && parsed.username().is_empty()
      && parsed.password().is_none())
  {
    return String::new();
  }

  if matches!(
    parsed.scheme(),
    "github" | "gitlab" | "bitbucket" | "gist" | "file"
  ) {
    return version_from_hash(&git_url);
  }
  let hostname = parsed.host_str().unwrap_or_default();
  if !HOSTNAME.is_match(hostname) {
    return String::new();
  }
  let hostname = hostname.strip_prefix("www.").unwrap_or(hostname);
  let hash = parsed
    .fragment()
    .map_or(String::new(), |hash| format!("#{hash}"));
  let hash = decode_url_component(&hash).unwrap_or_else(|| hash.clone().into());
  let path = parsed.path();
  let hash = match hostname {
    "github.com" => {
      let mut parts = path.split('/').skip(1);
      let user = parts.next().unwrap_or_default();
      let project = parts.next().unwrap_or_default().trim_end_matches(".git");
      let kind = parts.next().unwrap_or_default();
      if user.is_empty() || project.is_empty() || (!kind.is_empty() && kind != "tree") {
        return String::new();
      }
      if kind == "tree" {
        format!("#{}", parts.next().unwrap_or_default())
      } else {
        hash.into_owned()
      }
    }
    "gitlab.com" => {
      let path = path.trim_start_matches('/');
      let Some((user, project)) = path.rsplit_once('/') else {
        return String::new();
      };
      if user.is_empty()
        || project.trim_end_matches(".git").is_empty()
        || path.contains("/-/")
        || path.contains("/archive.tar.gz")
      {
        return String::new();
      }
      hash.into_owned()
    }
    "bitbucket.org" => {
      let mut parts = path.split('/').skip(1);
      if parts.next().unwrap_or_default().is_empty()
        || parts
          .next()
          .unwrap_or_default()
          .trim_end_matches(".git")
          .is_empty()
        || parts.next() == Some("get")
      {
        return String::new();
      }
      hash.into_owned()
    }
    "gist.github.com" => {
      let mut parts = path.split('/').skip(1);
      let user = parts.next().unwrap_or_default();
      let project = parts.next().unwrap_or(user);
      if project.trim_end_matches(".git").is_empty() || parts.next() == Some("raw") {
        return String::new();
      }
      hash.into_owned()
    }
    _ => hash.into_owned(),
  };
  let version = version_from_hash(&hash);
  if version.is_empty() { hash } else { version }
}

fn version_from_hash(value: &str) -> String {
  value.split_once('#').map_or(String::new(), |(_, hash)| {
    hash
      .strip_prefix("semver:")
      .filter(|version| !version.is_empty())
      .unwrap_or(hash)
      .to_owned()
  })
}

fn decode_url_component(value: &str) -> Option<Cow<'_, str>> {
  let mut bytes = value.bytes();
  while let Some(byte) = bytes.next() {
    if byte == b'%'
      && (!bytes.next().is_some_and(|b| b.is_ascii_hexdigit())
        || !bytes.next().is_some_and(|b| b.is_ascii_hexdigit()))
    {
      return None;
    }
  }
  urlencoding::decode(value).ok()
}
