use std::{borrow::Cow, path::Path};

use cow_utils::CowUtils;

use crate::{ContextGuard, Result, RootSpelling, cacheable, with::AsConverter};

// Escape literal NULs before inserting root markers, including when portability is disabled.
// The escape has no trailing NUL, so escaped data cannot form a root marker.
const ESCAPE: &str = "\0escape";

/// Characters that end a path inside an identifier. A root only matches right after one of them
/// or at the start, so a root that occurs inside a longer path is never rewritten.
const DELIMITERS: [char; 8] = ['|', '!', '?', '=', ' ', '\t', '\n', '\r'];

#[cfg(not(windows))]
const ROOT_SEPARATORS: [char; 1] = ['/'];
#[cfg(windows)]
const ROOT_SEPARATORS: [char; 2] = ['/', '\\'];

/// A separator form an identifier can spell the root with, and the markers that stand for the
/// root in that form. Distinct markers preserve Windows separator forms.
struct RootForm {
  separator: &'static str,
  context: &'static str,
  realpath: &'static str,
}

impl RootForm {
  fn marker(&self, spelling: RootSpelling) -> &'static str {
    match spelling {
      RootSpelling::Context => self.context,
      RootSpelling::Realpath => self.realpath,
    }
  }
}

#[cfg(not(windows))]
const ROOT_FORMS: [RootForm; 1] = [RootForm {
  separator: "/",
  context: "\0root/\0",
  realpath: "\0realpath/\0",
}];
#[cfg(windows)]
const ROOT_FORMS: [RootForm; 3] = [
  RootForm {
    separator: "\\\\",
    context: "\0root\\\\\0",
    realpath: "\0realpath\\\\\0",
  },
  RootForm {
    separator: "\\",
    context: "\0root\\\0",
    realpath: "\0realpath\\\0",
  },
  RootForm {
    separator: "/",
    context: "\0root/\0",
    realpath: "\0realpath/\0",
  },
];

/// Returns the rest of `text` after `root` spelled with `separator`, including the separator that
/// follows the root.
fn strip_root<'a>(text: &'a str, root: &str, separator: &str) -> Option<&'a str> {
  let mut rest = text;
  for (index, component) in root.split(ROOT_SEPARATORS).enumerate() {
    if index > 0 {
      rest = rest.strip_prefix(separator)?;
    }
    rest = rest.strip_prefix(component)?;
  }
  rest.strip_prefix(separator)
}

/// Returns the marker for the first root form `text` starts with, and the rest of `text`.
fn strip_any_root<'a>(
  text: &'a str,
  roots: &[(RootSpelling, Cow<'_, str>)],
) -> Option<(&'static str, &'a str)> {
  ROOT_FORMS.iter().find_map(|form| {
    roots.iter().find_map(|(spelling, root)| {
      strip_root(text, root, form.separator).map(|rest| (form.marker(*spelling), rest))
    })
  })
}

fn push_root(out: &mut String, root: &str, separator: &str) {
  for (index, component) in root.split(ROOT_SEPARATORS).enumerate() {
    if index > 0 {
      out.push_str(separator);
    }
    out.push_str(component);
  }
  out.push_str(separator);
}

fn trimmed_root(root: &Path) -> Cow<'_, str> {
  match root.to_string_lossy() {
    Cow::Borrowed(root) => Cow::Borrowed(root.trim_end_matches(ROOT_SEPARATORS)),
    Cow::Owned(root) => Cow::Owned(root.trim_end_matches(ROOT_SEPARATORS).to_owned()),
  }
}

/// Replaces whole root prefixes in identifiers while preserving paths outside the root.
#[cacheable(crate=crate, hashable)]
pub struct PortableString {
  content: String,
}

impl PortableString {
  /// Converts a typed context identifier by its [`crate::ContextIdentifierLayout`]: regex tokens are
  /// kept verbatim and each context path is converted on its own. Other identifiers convert as
  /// [`Self::new`].
  pub fn from_identifier(content: &str, guard: &ContextGuard) -> Self {
    let Some(layout) = guard
      .project_root()
      .and_then(|_| guard.context_identifier_layout(content))
    else {
      return Self::new(content, guard);
    };
    let mut cuts = layout
      .paths
      .iter()
      .copied()
      .chain(
        layout
          .regexes
          .iter()
          .flat_map(|range| [range.start, range.end]),
      )
      .collect::<Vec<_>>();
    if layout.regexes.iter().any(|range| range.is_empty())
      || cuts.iter().any(|cut| !content.is_char_boundary(*cut))
    {
      return Self::new(content, guard);
    }
    cuts.push(content.len());
    cuts.sort_unstable();
    cuts.dedup();
    let mut converted = String::with_capacity(content.len());
    let mut start = 0;
    for end in cuts {
      let segment = &content[start..end];
      if layout
        .regexes
        .iter()
        .any(|range| range.start <= start && end <= range.end)
      {
        converted.push_str(&segment.cow_replace('\0', ESCAPE));
      } else {
        converted.push_str(&Self::new(segment, guard).content);
      }
      start = end;
    }
    Self { content: converted }
  }

  pub fn new(content: &str, guard: &ContextGuard) -> Self {
    let roots = guard
      .root_spellings()
      .map(|(spelling, root)| (spelling, trimmed_root(root)))
      .collect::<Vec<_>>();
    if roots.is_empty() {
      return Self {
        content: content.cow_replace('\0', ESCAPE).into_owned(),
      };
    }
    let mut converted = String::with_capacity(content.len());
    let mut rest = content;
    loop {
      let path = match strip_any_root(rest, &roots) {
        Some((marker, suffix)) => {
          converted.push_str(marker);
          suffix
        }
        None => {
          let token = rest.split(DELIMITERS).next().unwrap_or_default();
          if Path::new(token).is_absolute() {
            guard.mark_outside_root();
          }
          rest
        }
      };
      let end = path.find(DELIMITERS).unwrap_or(path.len());
      converted.push_str(&path[..end].cow_replace('\0', ESCAPE));
      let Some(delimiter) = path[end..].chars().next() else {
        break;
      };
      converted.push(delimiter);
      rest = &path[end + delimiter.len_utf8()..];
    }
    Self { content: converted }
  }

  pub fn into_path_string(self, guard: &ContextGuard) -> String {
    if !self.content.contains('\0') {
      return self.content;
    }
    let mut restored = String::with_capacity(self.content.len());
    let mut rest = self.content.as_str();
    while let Some(index) = rest.find('\0') {
      restored.push_str(&rest[..index]);
      rest = &rest[index..];
      if let Some(after) = rest.strip_prefix(ESCAPE) {
        restored.push('\0');
        rest = after;
        continue;
      }
      let marker = ROOT_FORMS.iter().find_map(|form| {
        [RootSpelling::Context, RootSpelling::Realpath]
          .into_iter()
          .map(|spelling| (form, spelling, form.marker(spelling)))
          .find(|(_, _, marker)| rest.starts_with(marker))
      });
      let Some((form, spelling, marker)) = marker else {
        restored.push('\0');
        rest = &rest[1..];
        continue;
      };
      match guard.root(spelling) {
        Some(root) => push_root(&mut restored, &trimmed_root(root), form.separator),
        None => restored.push_str(marker),
      }
      rest = &rest[marker.len()..];
    }
    restored.push_str(rest);
    restored
  }
}

impl<T> AsConverter<T> for PortableString
where
  T: From<String> + AsRef<str>,
{
  fn serialize(data: &T, guard: &ContextGuard) -> Result<Self>
  where
    Self: Sized,
  {
    Ok(Self::new(data.as_ref(), guard))
  }

  fn deserialize(self, guard: &ContextGuard) -> Result<T> {
    Ok(T::from(self.into_path_string(guard)))
  }
}
