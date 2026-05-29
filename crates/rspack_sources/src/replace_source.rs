use std::{
  borrow::Cow,
  cell::RefCell,
  hash::{Hash, Hasher},
  sync::Arc,
};

use rustc_hash::FxHashMap as HashMap;

use crate::{
  BoxSource, MapOptions, Mapping, OriginalLocation, OriginalSource, Source, SourceExt, SourceMap,
  SourceValue,
  helpers::{Chunks, GeneratedInfo, StreamChunks, TextSpan, get_map},
  linear_map::LinearMap,
  object_pool::ObjectPool,
  source_content_lines::SourceContentLines,
};

/// Decorates a Source with replacements and insertions of source code,
/// usually used in dependencies
///
/// - [webpack-sources docs](https://github.com/webpack/webpack-sources/#replacesource).
///
/// ```
/// use rspack_sources::{OriginalSource, ReplaceSource, Source};
///
/// let code = "hello world\n";
/// let mut source = ReplaceSource::new(OriginalSource::new(code, "file.txt"));
///
/// source.insert_static(0, "start1\n", None);
/// source.replace_static(0, 0, "start2\n", None);
/// source.replace_static(999, 10000, "end2", None);
/// source.insert_static(888, "end1\n", None);
/// source.replace_static(0, 999, "replaced!\n", Some("whole"));
///
/// assert_eq!(
///   source.source().into_string_lossy(),
///   "start1\nstart2\nreplaced!\nend1\nend2"
/// );
/// ```
pub struct ReplaceSource {
  inner: BoxSource,
  replacements: Vec<Replacement>,
}

/// Enforce replacement order when two replacement start and end are both equal
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ReplacementEnforce {
  /// pre
  Pre,
  /// normal
  #[default]
  Normal,
  /// post
  Post,
}

/// A single text replacement in a [ReplaceSource].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Replacement {
  start: u32,
  end: u32,
  content: Cow<'static, str>,
  name: Option<Cow<'static, str>>,
  enforce: ReplacementEnforce,
  insertion_order: u32,
}

impl Replacement {
  /// Get the start offset.
  pub fn start(&self) -> u32 {
    self.start
  }

  /// Get the end offset.
  pub fn end(&self) -> u32 {
    self.end
  }

  /// Get the replacement content.
  pub fn content(&self) -> &str {
    &self.content
  }

  /// Get the replacement name.
  pub fn name(&self) -> Option<&str> {
    self.name.as_deref()
  }

  /// Get the replacement enforce order.
  pub fn enforce(&self) -> ReplacementEnforce {
    self.enforce
  }
}

impl Ord for Replacement {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    (self.start, self.end, self.enforce, self.insertion_order).cmp(&(
      other.start,
      other.end,
      other.enforce,
      other.insertion_order,
    ))
  }
}

impl PartialOrd for Replacement {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    Some(self.cmp(other))
  }
}

impl ReplaceSource {
  /// Create a [ReplaceSource].
  pub fn new<T: SourceExt>(source: T) -> Self {
    Self {
      inner: source.boxed(),
      replacements: Vec::new(),
    }
  }

  /// Get the inner source.
  pub fn inner(&self) -> &BoxSource {
    &self.inner
  }

  /// Get the sorted list of replacements.
  pub fn replacements(&self) -> &[Replacement] {
    &self.replacements
  }

  /// Insert a content at start.
  pub fn insert(&mut self, start: u32, content: String, name: Option<String>) {
    self.replace(start, start, content, name)
  }

  /// Insert a static string content at start.
  ///
  /// This method is optimized for `&'static str` inputs, avoiding unnecessary
  /// heap allocations by using `Cow::Borrowed` internally. Use this when you
  /// have string literals or other static strings that don't need to be owned.
  ///
  /// # Performance
  ///
  /// Prefer this over `insert()` when working with string literals, as it
  /// avoids cloning the string content.
  ///
  /// # Example
  ///
  /// ```no_run
  /// use rspack_sources::{OriginalSource, ReplaceSource};
  ///
  /// let mut source = ReplaceSource::new(OriginalSource::new("hello", "file.txt"));
  /// source.insert_static(0, "prefix: ", None);
  /// ```
  pub fn insert_static(&mut self, start: u32, content: &'static str, name: Option<&'static str>) {
    self.replace_static_with_enforce(start, start, content, name, ReplacementEnforce::Normal)
  }

  /// Insert a content at start, with ReplacementEnforce.
  pub fn insert_with_enforce(
    &mut self,
    start: u32,
    content: String,
    name: Option<String>,
    enforce: ReplacementEnforce,
  ) {
    self.replace_with_enforce(start, start, content, name, enforce)
  }

  /// Insert a static string content at start, with ReplacementEnforce.
  ///
  /// This method is optimized for `&'static str` inputs, avoiding unnecessary
  /// heap allocations by using `Cow::Borrowed` internally. Use this when you
  /// have string literals or other static strings that don't need to be owned.
  ///
  /// # Performance
  ///
  /// Prefer this over `insert_with_enforce()` when working with string literals,
  /// as it avoids cloning the string content.
  ///
  /// # Example
  ///
  /// ```no_run
  /// use rspack_sources::{OriginalSource, ReplaceSource, ReplacementEnforce};
  ///
  /// let mut source = ReplaceSource::new(OriginalSource::new("hello", "file.txt"));
  /// source.insert_static_with_enforce(0, "prefix: ", None, ReplacementEnforce::Pre);
  /// ```
  pub fn insert_static_with_enforce(
    &mut self,
    start: u32,
    content: &'static str,
    name: Option<&'static str>,
    enforce: ReplacementEnforce,
  ) {
    self.replace_static_with_enforce(start, start, content, name, enforce)
  }

  /// Create a replacement with content at `[start, end)`.
  pub fn replace(&mut self, start: u32, end: u32, content: String, name: Option<String>) {
    self.replace_with_enforce(start, end, content, name, ReplacementEnforce::Normal);
  }

  /// Create a replacement with static string content at `[start, end)`.
  ///
  /// This method is optimized for `&'static str` inputs, avoiding unnecessary
  /// heap allocations by using `Cow::Borrowed` internally. Use this when you
  /// have string literals or other static strings that don't need to be owned.
  ///
  /// # Performance
  ///
  /// Prefer this over `replace()` when working with string literals, as it
  /// avoids cloning the string content.
  ///
  /// # Example
  ///
  /// ```no_run
  /// use rspack_sources::{OriginalSource, ReplaceSource};
  ///
  /// let mut source = ReplaceSource::new(OriginalSource::new("hello world", "file.txt"));
  /// source.replace_static(0, 5, "hi", None);
  /// ```
  pub fn replace_static(
    &mut self,
    start: u32,
    end: u32,
    content: &'static str,
    name: Option<&'static str>,
  ) {
    self.replace_static_with_enforce(start, end, content, name, ReplacementEnforce::Normal)
  }

  /// Create a replacement with content at `[start, end)`, with ReplacementEnforce.
  pub fn replace_with_enforce(
    &mut self,
    start: u32,
    end: u32,
    content: String,
    name: Option<String>,
    enforce: ReplacementEnforce,
  ) {
    self.add_replacement(Replacement {
      start,
      end,
      content: content.into(),
      name: name.map(Into::into),
      enforce,
      insertion_order: self.replacements.len() as u32,
    });
  }

  /// Create a replacement with static string content at `[start, end)`, with ReplacementEnforce.
  ///
  /// This method is optimized for `&'static str` inputs, avoiding unnecessary
  /// heap allocations by using `Cow::Borrowed` internally. Use this when you
  /// have string literals or other static strings that don't need to be owned.
  ///
  /// # Performance
  ///
  /// Prefer this over `replace_with_enforce()` when working with string literals,
  /// as it avoids cloning the string content.
  ///
  /// # Example
  ///
  /// ```no_run
  /// use rspack_sources::{OriginalSource, ReplaceSource, ReplacementEnforce};
  ///
  /// let mut source = ReplaceSource::new(OriginalSource::new("hello world", "file.txt"));
  /// source.replace_static_with_enforce(0, 5, "hi", None, ReplacementEnforce::Pre);
  /// ```
  pub fn replace_static_with_enforce(
    &mut self,
    start: u32,
    end: u32,
    content: &'static str,
    name: Option<&'static str>,
    enforce: ReplacementEnforce,
  ) {
    self.add_replacement(Replacement {
      start,
      end,
      content: Cow::Borrowed(content),
      name: name.map(Cow::Borrowed),
      enforce,
      insertion_order: self.replacements.len() as u32,
    });
  }

  /// Internal helper method to add a replacement to the sorted list.
  ///
  /// This method maintains the replacements in sorted order for efficient
  /// processing. It uses binary search to find the correct insertion point
  /// when needed, or appends to the end if the replacement comes after all
  /// existing ones.
  #[inline]
  fn add_replacement(&mut self, replacement: Replacement) {
    if let Some(last) = self.replacements.last() {
      let cmp = replacement.cmp(last);
      if cmp == std::cmp::Ordering::Greater || cmp == std::cmp::Ordering::Equal {
        self.replacements.push(replacement);
      } else {
        let insert_at = self
          .replacements
          .binary_search_by(|other| other.cmp(&replacement))
          .unwrap_or_else(|e| e);
        self.replacements.insert(insert_at, replacement);
      }
    } else {
      self.replacements.push(replacement);
    }
  }
}

impl Source for ReplaceSource {
  fn source(&self) -> SourceValue<'_> {
    if self.replacements.is_empty() {
      return self.inner.source();
    }

    let mut string = String::with_capacity(self.size());
    self.rope(&mut |chunk| {
      string.push_str(chunk);
    });
    SourceValue::String(Cow::Owned(string))
  }

  #[allow(unsafe_code)]
  fn rope<'a>(&'a self, on_chunk: &mut dyn FnMut(&'a str)) {
    if self.replacements.is_empty() {
      return self.inner.rope(on_chunk);
    }

    let mut pos: usize = 0;
    let mut replacement_idx: usize = 0;
    let mut replacement_end: Option<usize> = None;
    let mut next_replacement: Option<usize> = self
      .replacements
      .get(replacement_idx)
      .map(|repl| repl.start as usize);

    self.inner.rope(&mut |chunk| {
      let mut chunk_pos = 0;
      let end_pos = pos + chunk.len();

      // Skip over when it has been replaced
      if let Some(replacement_end) =
        replacement_end.filter(|replacement_end| *replacement_end > pos)
      {
        // Skip over the whole chunk
        if replacement_end >= end_pos {
          pos = end_pos;
          return;
        }
        // Partially skip over chunk
        chunk_pos = replacement_end - pos;
        pos += chunk_pos;
      }

      // Is a replacement in the chunk?
      while let Some(next_replacement_pos) =
        next_replacement.filter(|next_replacement_pos| *next_replacement_pos < end_pos)
      {
        if next_replacement_pos > pos {
          // Emit chunk until replacement
          let offset = next_replacement_pos - pos;
          let chunk_slice = unsafe { chunk.get_unchecked(chunk_pos..(chunk_pos + offset)) };
          on_chunk(chunk_slice);
          chunk_pos += offset;
          pos = next_replacement_pos;
        }
        // Insert replacement content split into chunks by lines
        let replacement = unsafe { self.replacements.get_unchecked(replacement_idx) };
        on_chunk(&replacement.content);

        // Remove replaced content by settings this variable
        replacement_end = if let Some(replacement_end) = replacement_end {
          Some(replacement_end.max(replacement.end as usize))
        } else {
          Some(replacement.end as usize)
        };

        // Move to next replacement
        replacement_idx += 1;
        next_replacement = self
          .replacements
          .get(replacement_idx)
          .map(|repl| repl.start as usize);

        // Skip over when it has been replaced
        let offset =
          chunk.len() as i64 - end_pos as i64 + replacement_end.unwrap() as i64 - chunk_pos as i64;
        if offset > 0 {
          // Skip over whole chunk
          if replacement_end.is_some_and(|replacement_end| replacement_end >= end_pos) {
            pos = end_pos;
            return;
          }

          // Partially skip over chunk
          chunk_pos += offset as usize;
          pos += offset as usize;
        }
      }

      // Emit remaining chunk
      if chunk_pos < chunk.len() {
        on_chunk(unsafe { chunk.get_unchecked(chunk_pos..) });
      }
      pos = end_pos;
    });

    // Handle remaining replacements one by one
    while replacement_idx < self.replacements.len() {
      let replacement = unsafe { self.replacements.get_unchecked(replacement_idx) };
      let content = &replacement.content;
      on_chunk(content);
      replacement_idx += 1;
    }
  }

  fn buffer(&self) -> Cow<'_, [u8]> {
    self.source().into_bytes()
  }

  fn size(&self) -> usize {
    let inner_source_size = self.inner.size();

    if self.replacements.is_empty() {
      return inner_source_size;
    }

    // Simulate the replacement process to calculate accurate size
    let mut size = inner_source_size;
    let mut inner_pos = 0u32;

    for replacement in self.replacements.iter() {
      // Add original content before replacement
      if inner_pos < replacement.start {
        // This content is already counted in inner_source_size, so no change needed
      }
      if replacement.start as usize >= inner_source_size {
        size += replacement.content.len();
        continue;
      }

      // Handle the replacement itself
      let original_length = replacement
        .end
        .saturating_sub(replacement.start.max(inner_pos)) as usize;
      let replacement_length = replacement.content.len();

      // Subtract original content length and add replacement content length
      size = size
        .saturating_sub(original_length)
        .saturating_add(replacement_length);

      // Move position forward, handling overlaps
      inner_pos = inner_pos.max(replacement.end);
    }

    size
  }

  fn map(&self, object_pool: &ObjectPool, options: &crate::MapOptions) -> Option<SourceMap> {
    let replacements = &self.replacements;
    if replacements.is_empty() {
      return self.inner.map(object_pool, options);
    }
    let chunks = self.stream_chunks();
    get_map(object_pool, chunks.as_ref(), options)
  }

  fn to_writer(&self, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
    let mut result = Ok(());
    self.rope(&mut |chunk| {
      if result.is_err() {
        return;
      }
      result = writer.write_all(chunk.as_bytes());
    });
    result
  }
}

impl std::fmt::Debug for ReplaceSource {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
    let indent = f.width().unwrap_or(0);
    let indent_str = format!("{:indent$}", "", indent = indent);

    writeln!(f, "{indent_str}{{")?;
    writeln!(f, "{indent_str}  let mut source = ReplaceSource::new(")?;
    writeln!(f, "{:indent$?}", &self.inner, indent = indent + 4)?;
    writeln!(f, "{indent_str}  );")?;
    for repl in self.replacements.iter() {
      match repl.enforce {
        ReplacementEnforce::Pre => {
          writeln!(
            f,
            "{indent_str}  source.replace_with_enforce({:#?}, {:#?}, {:#?}, {:#?}, ReplacementEnforce::Pre);",
            repl.start, repl.end, repl.content, repl.name
          )?;
        }
        ReplacementEnforce::Normal => {
          writeln!(
            f,
            "{indent_str}  source.replace({:#?}, {:#?}, {:#?}, {:#?});",
            repl.start, repl.end, repl.content, repl.name
          )?;
        }
        ReplacementEnforce::Post => {
          writeln!(
            f,
            "{indent_str}  source.replace_with_enforce({:#?}, {:#?}, {:#?}, {:#?}, ReplacementEnforce::Post);",
            repl.start, repl.end, repl.content, repl.name
          )?;
        }
      }
    }
    writeln!(f, "{indent_str}  source.boxed()")?;
    write!(f, "{indent_str}}}")
  }
}

enum SourceContent<'object_pool> {
  Raw(Arc<str>),
  Lines(SourceContentLines<'object_pool>),
}

fn check_content_at_position(
  lines: &SourceContentLines,
  line: u32,
  column: u32,
  expected: &str,
) -> bool {
  if let Some(line) = lines.get(line as usize - 1) {
    line
      .substring(column as usize, usize::MAX)
      .starts_with(expected)
  } else {
    false
  }
}

#[inline]
fn for_each_line<'a>(source: &'a str, mut on_line: impl FnMut(&'a str, bool)) {
  let mut remaining = source;
  while !remaining.is_empty() {
    match memchr::memchr(b'\n', remaining.as_bytes()) {
      Some(pos) => {
        #[allow(unsafe_code)]
        // SAFETY: `pos` points to an ASCII newline found in a valid UTF-8 str,
        // so both byte ranges are valid UTF-8 boundaries.
        let (line, next) = unsafe {
          (
            remaining.get_unchecked(..pos + 1),
            remaining.get_unchecked(pos + 1..),
          )
        };
        remaining = next;
        on_line(line, true);
      }
      None => {
        on_line(remaining, false);
        break;
      }
    }
  }
}

struct ReplaceSourceChunks<'a> {
  is_original_source: bool,
  chunks: Box<dyn Chunks + 'a>,
  replacements: &'a [Replacement],
}

impl<'a> ReplaceSourceChunks<'a> {
  pub fn new(source: &'a ReplaceSource) -> Self {
    let is_original_source = source.inner.as_ref().as_any().is::<OriginalSource>();
    Self {
      is_original_source,
      chunks: source.inner.stream_chunks(),
      replacements: &source.replacements,
    }
  }
}

impl Chunks for ReplaceSourceChunks<'_> {
  fn stream<'a>(
    &'a self,
    object_pool: &'a ObjectPool,
    options: &MapOptions,
    on_chunk: crate::helpers::OnChunk<'_, 'a>,
    on_source: crate::helpers::OnSource<'_, 'a>,
    on_name: crate::helpers::OnName<'_, 'a>,
  ) -> crate::helpers::GeneratedInfo {
    let on_name = RefCell::new(on_name);
    let repls = &self.replacements;
    let mut pos: u32 = 0;
    let mut i: usize = 0;
    let mut replacement_end: Option<u32> = None;
    let mut next_replacement = (i < repls.len()).then(|| repls[i].start);
    let mut generated_line_offset: i64 = 0;
    let mut generated_column_offset: i64 = 0;
    let mut generated_column_offset_line = 0;
    let source_content_lines: RefCell<LinearMap<Option<SourceContent>>> =
      RefCell::new(LinearMap::default());
    let name_mapping: RefCell<HashMap<Cow<str>, u32>> = RefCell::new(HashMap::default());
    let name_index_mapping: RefCell<LinearMap<u32>> = RefCell::new(LinearMap::default());

    // check if source_content[line][col] is equal to expect
    // Why this is needed?
    //
    // For example, there is an source_map like (It's OriginalSource)
    //    source_code: "jsx || tsx"
    //    mappings:    ↑
    //    target_code: "jsx || tsx"
    // If replace || to &&, there will be some new mapping information
    //    source_code: "jsx || tsx"
    //    mappings:    ↑    ↑  ↑
    //    target_code: "jsx && tsx"
    //
    // In this case, because source_content[line][col] is equal to target, we can split this mapping correctly,
    // Therefore, we can add some extra mappings for this replace operation.
    //
    // But for this example, source_content[line][col] is not equal to target (It's SourceMapSource)
    //    source_code: "<div />"
    //    mappings:    ↑
    //    target_code: "jsx || tsx"
    // If replace || to && also, then
    //    source_code: "<div />"
    //    mappings:    ↑
    //    target_code: "jsx && tsx"
    //
    // In this case, we can't split this mapping.
    // webpack-sources also have this function, refer https://github.com/webpack/webpack-sources/blob/main/lib/ReplaceSource.js#L158
    let check_original_content =
      |source_index: u32, line: u32, column: u32, expected_chunk: &str| {
        if let Some(Some(source_content)) = source_content_lines.borrow_mut().get_mut(&source_index)
        {
          match source_content {
            SourceContent::Raw(source) => {
              let lines = SourceContentLines::new(object_pool, source.clone());
              let matched = check_content_at_position(&lines, line, column, expected_chunk);
              *source_content = SourceContent::Lines(lines);
              matched
            }
            SourceContent::Lines(lines) => {
              check_content_at_position(lines, line, column, expected_chunk)
            }
          }
        } else {
          false
        }
      };

    let result = self.chunks.stream(
      object_pool,
      &MapOptions {
        columns: options.columns,
        final_source: false,
      },
      &mut |chunk, mut mapping| {
        // SAFETY: final_source is false in ReplaceSource
        let chunk = chunk.unwrap();
        let mut chunk_pos = 0;
        let end_pos = pos + chunk.len() as u32;
        // Skip over when it has been replaced
        if let Some(replacement_end) =
          replacement_end.filter(|replacement_end| *replacement_end > pos)
        {
          // Skip over the whole chunk
          if replacement_end >= end_pos {
            let line = mapping.generated_line as i64 + generated_line_offset;
            if chunk.ends_with('\n') {
              generated_line_offset -= 1;
              if generated_column_offset_line == line {
                // undo exiting corrections form the current line
                generated_column_offset += mapping.generated_column as i64;
              }
            } else if generated_column_offset_line == line {
              generated_column_offset -= chunk.utf16_len() as i64;
            } else {
              generated_column_offset = -(chunk.utf16_len() as i64);
              generated_column_offset_line = line;
            }
            pos = end_pos;
            return;
          }
          // Partially skip over chunk
          chunk_pos = replacement_end - pos;
          if let Some(original) = mapping.original.as_mut() {
            if self.is_original_source
              || check_original_content(
                original.source_index,
                original.original_line,
                original.original_column,
                chunk.slice_to(chunk_pos as usize).as_str(),
              )
            {
              original.original_column += chunk_pos;
            }
          }
          pos += chunk_pos;
          let chunk_utf16_pos = chunk.slice_to(chunk_pos as usize).utf16_len();
          let line = mapping.generated_line as i64 + generated_line_offset;
          if generated_column_offset_line == line {
            generated_column_offset -= chunk_utf16_pos as i64;
          } else {
            generated_column_offset = -(chunk_utf16_pos as i64);
            generated_column_offset_line = line;
          }
          mapping.generated_column += chunk_utf16_pos as u32;
        }

        // Is a replacement in the chunk?
        while let Some(next_replacement_pos) =
          next_replacement.filter(|next_replacement_pos| *next_replacement_pos < end_pos)
        {
          let mut line = mapping.generated_line as i64 + generated_line_offset;
          if next_replacement_pos > pos {
            // Emit chunk until replacement
            let offset = next_replacement_pos - pos;
            let chunk_slice = chunk.slice(chunk_pos as usize, (chunk_pos + offset) as usize);
            let chunk_slice_utf16_offset = chunk_slice.utf16_len() as u32;
            on_chunk(
              Some(chunk_slice),
              Mapping {
                generated_line: line as u32,
                generated_column: ((mapping.generated_column as i64)
                  + if line == generated_column_offset_line {
                    generated_column_offset
                  } else {
                    0
                  }) as u32,
                original: if self.is_original_source {
                  mapping.original
                } else {
                  mapping.original.as_ref().map(|original| OriginalLocation {
                    source_index: original.source_index,
                    original_line: original.original_line,
                    original_column: original.original_column,
                    name_index: original
                      .name_index
                      .and_then(|name_index| name_index_mapping.borrow().get(&name_index).copied()),
                  })
                },
              },
            );
            mapping.generated_column += chunk_slice_utf16_offset;
            chunk_pos += offset;
            pos = next_replacement_pos;
            if let Some(original) = mapping.original.as_mut() {
              if self.is_original_source
                || check_original_content(
                  original.source_index,
                  original.original_line,
                  original.original_column,
                  chunk_slice.as_str(),
                )
              {
                original.original_column += chunk_slice_utf16_offset;
              }
            }
          }
          // Insert replacement content split into chunks by lines
          #[allow(unsafe_code)]
          // SAFETY: The safety of this operation relies on the fact that the `ReplaceSource` type will not delete the `replacements` during its entire lifetime.
          let repl = &repls[i];

          let mut replacement_name_index = mapping
            .original
            .as_ref()
            .and_then(|original| original.name_index);
          if mapping.original.is_some() {
            if let Some(name) = repl.name.as_ref() {
              let mut name_mapping = name_mapping.borrow_mut();
              let mut global_index = name_mapping.get(name.as_ref()).copied();
              if global_index.is_none() {
                let len = name_mapping.len() as u32;
                name_mapping.insert(Cow::Borrowed(name), len);
                on_name.borrow_mut()(len, Cow::Borrowed(name));
                global_index = Some(len);
              }
              replacement_name_index = global_index;
            }
          }
          let content = repl.content.as_ref();
          let content_is_ascii = content.is_ascii();
          for_each_line(content, |content_line, ends_with_newline| {
            let content_chunk = TextSpan::with_ascii(content_line, content_is_ascii);
            on_chunk(
              Some(content_chunk),
              Mapping {
                generated_line: line as u32,
                generated_column: ((mapping.generated_column as i64)
                  + if line == generated_column_offset_line {
                    generated_column_offset
                  } else {
                    0
                  }) as u32,
                original: if mapping.original.and_then(|original| original.name_index)
                  == replacement_name_index
                {
                  mapping.original
                } else {
                  mapping.original.map(|original| OriginalLocation {
                    source_index: original.source_index,
                    original_line: original.original_line,
                    original_column: original.original_column,
                    name_index: replacement_name_index,
                  })
                },
              },
            );
            // Only the first chunk has name assigned
            replacement_name_index = None;

            if !ends_with_newline {
              let content_utf16_len = content_chunk.utf16_len() as i64;
              if generated_column_offset_line == line {
                generated_column_offset += content_utf16_len;
              } else {
                generated_column_offset = content_utf16_len;
                generated_column_offset_line = line;
              }
            } else {
              generated_line_offset += 1;
              line += 1;
              generated_column_offset = -(mapping.generated_column as i64);
              generated_column_offset_line = line;
            }
          });

          // Remove replaced content by settings this variable
          replacement_end = if let Some(replacement_end) = replacement_end {
            Some(replacement_end.max(repl.end))
          } else {
            Some(repl.end)
          };

          // Move to next replacement
          i += 1;
          next_replacement = if i < repls.len() {
            Some(repls[i].start)
          } else {
            None
          };

          // Skip over when it has been replaced
          let offset = chunk.len() as i64 - end_pos as i64 + replacement_end.unwrap() as i64
            - chunk_pos as i64;
          if offset > 0 {
            // Skip over whole chunk
            if replacement_end.is_some_and(|replacement_end| replacement_end >= end_pos) {
              let line = mapping.generated_line as i64 + generated_line_offset;
              if chunk.ends_with('\n') {
                generated_line_offset -= 1;
                if generated_column_offset_line == line {
                  // undo exiting corrections form the current line
                  generated_column_offset += mapping.generated_column as i64;
                }
              } else if generated_column_offset_line == line {
                let remaining_chunk_utf16_len =
                  chunk.slice_from(chunk_pos as usize).utf16_len() as i64;
                generated_column_offset -= remaining_chunk_utf16_len;
              } else {
                generated_column_offset =
                  -(chunk.slice_from(chunk_pos as usize).utf16_len() as i64);
                generated_column_offset_line = line;
              }
              pos = end_pos;
              return;
            }

            // Partially skip over chunk
            let line = mapping.generated_line as i64 + generated_line_offset;
            if let Some(original) = mapping.original.as_mut() {
              if self.is_original_source
                || check_original_content(
                  original.source_index,
                  original.original_line,
                  original.original_column,
                  chunk
                    .slice(chunk_pos as usize, (chunk_pos + offset as u32) as usize)
                    .as_str(),
                )
              {
                original.original_column += offset as u32;
              }
            }

            let utf16_offset = chunk
              .slice(chunk_pos as usize, (chunk_pos + offset as u32) as usize)
              .utf16_len() as i64;
            chunk_pos += offset as u32;
            pos += offset as u32;

            if generated_column_offset_line == line {
              generated_column_offset -= utf16_offset;
            } else {
              generated_column_offset = -utf16_offset;
              generated_column_offset_line = line;
            }
            mapping.generated_column += utf16_offset as u32;
          }
        }

        // Emit remaining chunk
        if (chunk_pos as usize) < chunk.len() {
          let chunk_slice = if chunk_pos == 0 {
            chunk
          } else {
            chunk.slice_from(chunk_pos as usize)
          };
          let line = mapping.generated_line as i64 + generated_line_offset;
          on_chunk(
            Some(chunk_slice),
            Mapping {
              generated_line: line as u32,
              generated_column: ((mapping.generated_column as i64)
                + if line == generated_column_offset_line {
                  generated_column_offset
                } else {
                  0
                }) as u32,
              original: if self.is_original_source {
                mapping.original
              } else {
                mapping.original.as_ref().map(|original| OriginalLocation {
                  source_index: original.source_index,
                  original_line: original.original_line,
                  original_column: original.original_column,
                  name_index: original
                    .name_index
                    .and_then(|name_index| name_index_mapping.borrow().get(&name_index).copied()),
                })
              },
            },
          );
        }
        pos = end_pos;
      },
      &mut |source_index, source, source_content| {
        if !self.is_original_source {
          let mut source_content_lines = source_content_lines.borrow_mut();
          let lines =
            source_content.map(|source_content| SourceContent::Raw(source_content.clone()));
          source_content_lines.insert(source_index, lines);
        }
        on_source(source_index, source, source_content);
      },
      &mut |name_index, name| {
        let mut name_mapping = name_mapping.borrow_mut();
        let mut global_index = name_mapping.get(&name).copied();
        if global_index.is_none() {
          let len = name_mapping.len() as u32;
          name_mapping.insert(name.clone(), len);
          on_name.borrow_mut()(len, name);
          global_index = Some(len);
        }
        name_index_mapping
          .borrow_mut()
          .insert(name_index, global_index.unwrap());
      },
    );

    // Handle remaining replacements one by one
    let mut line = result.generated_line as i64 + generated_line_offset;
    while i < repls.len() {
      let content = repls[i].content.as_ref();
      let content_is_ascii = content.is_ascii();

      for_each_line(content, |content_line, ends_with_newline| {
        let content_chunk = TextSpan::with_ascii(content_line, content_is_ascii);
        on_chunk(
          Some(content_chunk),
          Mapping {
            generated_line: line as u32,
            generated_column: ((result.generated_column as i64)
              + if line == generated_column_offset_line {
                generated_column_offset
              } else {
                0
              }) as u32,
            original: None,
          },
        );

        // Handle line and column offset updates
        if !ends_with_newline {
          let content_utf16_len = content_chunk.utf16_len() as i64;
          // Last line of current replacement doesn't end with newline
          if generated_column_offset_line == line {
            generated_column_offset += content_utf16_len;
          } else {
            generated_column_offset = content_utf16_len;
            generated_column_offset_line = line;
          }
        } else {
          // Line ends with newline or not the last line
          line += 1;
          generated_column_offset = -(result.generated_column as i64);
          generated_column_offset_line = line;
        }
      });

      i += 1;
    }

    GeneratedInfo {
      generated_line: line as u32,
      generated_column: ((result.generated_column as i64)
        + if line == generated_column_offset_line {
          generated_column_offset
        } else {
          0
        }) as u32,
    }
  }
}

impl StreamChunks for ReplaceSource {
  fn stream_chunks<'a>(&'a self) -> Box<dyn Chunks + 'a> {
    Box::new(ReplaceSourceChunks::new(self))
  }
}

impl Clone for ReplaceSource {
  fn clone(&self) -> Self {
    Self {
      inner: self.inner.clone(),
      replacements: self.replacements.clone(),
    }
  }
}

impl Hash for ReplaceSource {
  fn hash<H: Hasher>(&self, state: &mut H) {
    "ReplaceSource".hash(state);
    // replacements are ordered, so when hashing,
    // skip fields (enforce and insertion_order) that are only used
    for repl in &self.replacements {
      repl.start.hash(state);
      repl.end.hash(state);
      repl.content.hash(state);
      repl.name.hash(state);
    }
    self.inner.hash(state);
  }
}

impl PartialEq for ReplaceSource {
  fn eq(&self, other: &Self) -> bool {
    self.inner.as_ref() == other.inner.as_ref() && self.replacements == other.replacements
  }
}

impl Eq for ReplaceSource {}

#[cfg(test)]
mod tests {
  use rustc_hash::FxHasher;

  use super::*;
  use crate::{
    OriginalSource, RawStringSource, ReplacementEnforce, SourceExt, SourceMapSource,
    SourceMapSourceOptions, source_map_source::WithoutOriginalOptions,
  };

  fn with_readable_mappings(sourcemap: &SourceMap) -> String {
    let mut first = true;
    let mut last_line = 0;
    sourcemap
      .decoded_mappings()
      .map(|token| {
        format!(
          "{}:{} ->{} {}:{}{}",
          if !first && token.generated_line == last_line {
            ", ".to_owned()
          } else {
            first = false;
            last_line = token.generated_line;
            format!("\n{}", token.generated_line)
          },
          token.generated_column,
          token
            .original
            .as_ref()
            .and_then(|original| sourcemap.get_source(original.source_index as usize))
            .map_or("".to_owned(), |source| format!(" [{source}]")),
          token
            .original
            .as_ref()
            .map(|original| original.original_line)
            .unwrap_or(!0),
          token
            .original
            .as_ref()
            .map(|original| original.original_column)
            .unwrap_or(!0),
          token
            .original
            .as_ref()
            .and_then(|original| original.name_index)
            .and_then(|name_index| sourcemap.get_name(name_index as usize))
            .map_or("".to_owned(), |source| format!(" ({source})")),
        )
      })
      .collect()
  }

  #[test]
  fn should_replace_correctly() {
    let line1 = "Hello World!";
    let line2 = "{}";
    let line3 = "Line 3";
    let line4 = "Line 4";
    let line5 = "Line 5";
    let code = [line1, line2, line3, line4, line5, "Last", "Line"].join("\n");
    let mut source = ReplaceSource::new(OriginalSource::new(code.as_str(), "file.txt"));

    let start_line3 = (line1.len() + line2.len() + 2) as u32;
    let start_line6 = (start_line3 as usize + line3.len() + line4.len() + line5.len() + 3) as u32;
    source.replace_static(start_line3, start_line6, "", None);
    source.replace_static(1, 5, "i ", None);
    source.replace_static(1, 5, "bye", None);
    source.replace_static(7, 8, "0000", None);
    source.insert_static((line1.len() + 2) as u32, "\n Multi Line\n", None);
    source.replace_static(start_line6 + 4, start_line6 + 5, " ", None);

    let result = source.source();
    let result_map = source
      .map(&ObjectPool::default(), &MapOptions::default())
      .unwrap();

    assert_eq!(
      code,
      r#"Hello World!
{}
Line 3
Line 4
Line 5
Last
Line"#
    );

    assert_eq!(
      result.into_string_lossy(),
      r#"Hi bye W0000rld!
{
 Multi Line
}
Last Line"#
    );

    assert_eq!(
      with_readable_mappings(&result_map),
      r#"
1:0 -> [file.txt] 1:0, :1 -> [file.txt] 1:1, :3 -> [file.txt] 1:5, :8 -> [file.txt] 1:7, :12 -> [file.txt] 1:8
2:0 -> [file.txt] 2:0, :1 -> [file.txt] 2:1
3:0 -> [file.txt] 2:1
4:0 -> [file.txt] 2:1
5:0 -> [file.txt] 6:0, :4 -> [file.txt] 6:4, :5 -> [file.txt] 7:0"#
    );

    let result_list_map = source
      .map(&ObjectPool::default(), &MapOptions::new(false))
      .unwrap();
    assert_eq!(
      with_readable_mappings(&result_list_map),
      r#"
1:0 -> [file.txt] 1:0
2:0 -> [file.txt] 2:0
3:0 -> [file.txt] 2:0
4:0 -> [file.txt] 2:0
5:0 -> [file.txt] 6:0"#
    );
  }

  #[test]
  fn should_replace_multiple_items_correctly() {
    let line1 = "Hello";
    let mut source = ReplaceSource::new(OriginalSource::new(
      ["Hello", "World!"].join("\n").as_str(),
      "file.txt",
    ));
    let original_code = source.source().into_string_lossy().into_owned();
    source.insert_static(0, "Message: ", None);
    source.replace_static(2, (line1.len() + 5) as u32, "y A", None);
    let result_text = source.source();
    let result_map = source
      .map(&ObjectPool::default(), &MapOptions::default())
      .unwrap();
    let result_list_map = source
      .map(&ObjectPool::default(), &MapOptions::new(false))
      .unwrap();

    assert_eq!(
      original_code,
      r#"Hello
World!"#
    );
    assert_eq!(result_text.into_string_lossy(), "Message: Hey Ad!");
    assert_eq!(
      with_readable_mappings(&result_map),
      r#"
1:0 -> [file.txt] 1:0, :11 -> [file.txt] 1:2, :14 -> [file.txt] 2:4"#
    );
    assert_eq!(
      with_readable_mappings(&result_list_map),
      r#"
1:0 -> [file.txt] 1:0"#
    );
    assert_eq!(result_map.mappings(), "AAAA,WAAE,GACE");
    assert_eq!(result_list_map.mappings(), "AAAA");
  }

  #[test]
  fn should_prepend_items_correctly() {
    let mut source = ReplaceSource::new(OriginalSource::new("Line 1", "file.txt"));
    source.insert_static(0, "Line -1\n", None);
    source.insert_static(0, "Line 0\n", None);

    let result_text = source.source();
    let result_map = source
      .map(&ObjectPool::default(), &MapOptions::default())
      .unwrap();
    let result_list_map = source
      .map(&ObjectPool::default(), &MapOptions::new(false))
      .unwrap();

    assert_eq!(result_text.into_string_lossy(), "Line -1\nLine 0\nLine 1");
    assert_eq!(
      with_readable_mappings(&result_map),
      r#"
1:0 -> [file.txt] 1:0
2:0 -> [file.txt] 1:0
3:0 -> [file.txt] 1:0"#
    );
    assert_eq!(
      with_readable_mappings(&result_list_map),
      r#"
1:0 -> [file.txt] 1:0
2:0 -> [file.txt] 1:0
3:0 -> [file.txt] 1:0"#
    );
  }

  #[test]
  fn should_prepend_items_with_replace_at_start_correctly() {
    let mut source = ReplaceSource::new(OriginalSource::new(
      ["Line 1", "Line 2"].join("\n").as_str(),
      "file.txt",
    ));
    source.insert_static(0, "Line 0\n", None);
    source.replace_static(0, 6, "Hello", None);
    let result_text = source.source();
    let result_map = source
      .map(&ObjectPool::default(), &MapOptions::default())
      .unwrap();
    let result_list_map = source
      .map(&ObjectPool::default(), &MapOptions::new(false))
      .unwrap();

    assert_eq!(
      result_text.into_string_lossy(),
      r#"Line 0
Hello
Line 2"#
    );
    assert_eq!(
      result_map.to_json(),
      r#"{"version":3,"sources":["file.txt"],"sourcesContent":["Line 1\nLine 2"],"names":[],"mappings":"AAAA;AAAA,KAAM;AACN"}"#
    );
    assert_eq!(
      result_list_map.to_json(),
      r#"{"version":3,"sources":["file.txt"],"sourcesContent":["Line 1\nLine 2"],"names":[],"mappings":"AAAA;AAAA;AACA"}"#
    );
  }

  #[test]
  fn should_append_items_correctly() {
    let line1 = "Line 1\n";
    let mut source = ReplaceSource::new(OriginalSource::new(line1, "file.txt"));
    source.insert_static((line1.len() + 1) as u32, "Line 2\n", None);
    let result_text = source.source();
    let result_map = source
      .map(&ObjectPool::default(), &MapOptions::default())
      .unwrap();
    let result_list_map = source
      .map(&ObjectPool::default(), &MapOptions::new(false))
      .unwrap();

    assert_eq!(result_text.into_string_lossy(), "Line 1\nLine 2\n");
    assert_eq!(
      result_map.to_json(),
      r#"{"version":3,"sources":["file.txt"],"sourcesContent":["Line 1\n"],"names":[],"mappings":"AAAA"}"#
    );
    assert_eq!(
      result_list_map.to_json(),
      r#"{"version":3,"sources":["file.txt"],"sourcesContent":["Line 1\n"],"names":[],"mappings":"AAAA"}"#
    );
  }

  #[test]
  fn should_produce_correct_source_map() {
    let bootstrap_code = "   var hello\n   var world\n";
    let mut source = ReplaceSource::new(OriginalSource::new(bootstrap_code, "file.js"));
    source.replace_static(7, 12, "h", Some("hello"));
    source.replace_static(20, 25, "w", Some("world"));
    let result_map = source
      .map(&ObjectPool::default(), &MapOptions::default())
      .expect("failed");

    let target_code = source.source();
    assert_eq!(target_code.into_string_lossy(), "   var h\n   var w\n");

    assert_eq!(
      with_readable_mappings(&result_map),
      r#"
1:0 -> [file.js] 1:0, :7 -> [file.js] 1:7 (hello), :8 -> [file.js] 1:12
2:0 -> [file.js] 2:0, :7 -> [file.js] 2:7 (world), :8 -> [file.js] 2:12"#
    );
    assert_eq!(
      result_map.to_json(),
      r#"{"version":3,"sources":["file.js"],"sourcesContent":["   var hello\n   var world\n"],"names":["hello","world"],"mappings":"AAAA,OAAOA,CAAK;AACZ,OAAOC,CAAK"}"#
    );
  }

  #[test]
  fn should_allow_replacements_at_the_start() {
    let map = SourceMap::from_slice(
      r#"{
        "version":3,
        "sources":["abc"],
        "names":["StaticPage","data","foo"],
        "mappings":";;AAAA,eAAe,SAASA,UAAT,OAA8B;AAAA,MAARC,IAAQ,QAARA,IAAQ;AAC3C,sBAAO;AAAA,cAAMA,IAAI,CAACC;AAAX,IAAP;AACD",
        "sourcesContent":["export default function StaticPage({ data }) {\nreturn <div>{data.foo}</div>\n}\n"],
        "file":"x"
      }"#.as_bytes(),
    ).unwrap();

    let code = r#"import { jsx as _jsx } from "react/jsx-runtime";
export var __N_SSG = true;
export default function StaticPage(_ref) {
  var data = _ref.data;
  return /*#__PURE__*/_jsx("div", {
    children: data.foo
  });
}"#;

    /*
      3:0 -> [abc] 1:0, :15 -> [abc] 1:15, :24 -> [abc] 1:24 (StaticPage), :34 -> [abc] 1:15, :41 -> [abc] 1:45
      4:0 -> [abc] 1:45, :6 -> [abc] 1:37 (data), :10 -> [abc] 1:45, :18 -> [abc] 1:37 (data), :22 -> [abc] 1:45
      5:0 -> [abc] 2:2, :22 -> [abc] 2:9
      6:0 -> [abc] 2:9, :14 -> [abc] 2:15 (data), :18 -> [abc] 2:19, :19 -> [abc] 2:20 (foo)
      7:0 -> [abc] 2:9, :4 -> [abc] 2:2
      8:0 -> [abc] 3:1
    */

    let mut source = ReplaceSource::new(SourceMapSource::new(WithoutOriginalOptions {
      value: code,
      name: "source.js",
      source_map: map,
    }));
    source.replace_static(0, 48, "", None);
    source.replace_static(49, 56, "", None);
    source.replace_static(76, 91, "", None);
    source.replace_static(
      165,
      169,
      "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)",
      None,
    );

    let target_code = source.source();
    let source_map = source
      .map(&ObjectPool::default(), &MapOptions::default())
      .unwrap();

    assert_eq!(
      target_code.into_string_lossy(),
      r#"
var __N_SSG = true;
function StaticPage(_ref) {
  var data = _ref.data;
  return /*#__PURE__*/(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)("div", {
    children: data.foo
  });
}"#
    );
    assert_eq!(source_map.get_name(0).unwrap(), "StaticPage");
    assert_eq!(source_map.get_name(1).unwrap(), "data");
    assert_eq!(source_map.get_name(2).unwrap(), "foo");
    assert_eq!(
      source_map.get_source_content(0).unwrap().as_ref(),
      r#"export default function StaticPage({ data }) {
return <div>{data.foo}</div>
}
"#
    );
    assert!(source_map.file().is_none());
    assert_eq!(source_map.get_source(0).unwrap(), "abc");

    assert_eq!(
      with_readable_mappings(&source_map),
      r#"
3:0 -> [abc] 1:15, :9 -> [abc] 1:24 (StaticPage), :19 -> [abc] 1:15, :26 -> [abc] 1:45
4:0 -> [abc] 1:45, :6 -> [abc] 1:37 (data), :10 -> [abc] 1:45, :18 -> [abc] 1:37 (data), :22 -> [abc] 1:45
5:0 -> [abc] 2:2, :22 -> [abc] 2:9
6:0 -> [abc] 2:9, :14 -> [abc] 2:15 (data), :18 -> [abc] 2:19, :19 -> [abc] 2:20 (foo)
7:0 -> [abc] 2:9, :4 -> [abc] 2:2
8:0 -> [abc] 3:1"#
    );
  }

  #[test]
  fn should_not_generate_invalid_mappings_when_replacing_multiple_lines_of_code() {
    let mut source = ReplaceSource::new(OriginalSource::new(
      r#"if (a;b;c) {
  a; b; c;
}"#,
      "document.js",
    ));
    source.replace_static(4, 9, "false", None);
    source.replace_static(12, 24, "", None);

    let target_code = source.source();
    let source_map = source
      .map(&ObjectPool::default(), &MapOptions::default())
      .unwrap();

    assert_eq!(target_code.into_string_lossy(), "if (false) {}");
    assert_eq!(
      with_readable_mappings(&source_map),
      r#"
1:0 -> [document.js] 1:0, :4 -> [document.js] 1:4, :9 -> [document.js] 1:9, :12 -> [document.js] 3:0"#
    );
    assert_eq!(
      source_map.to_json(),
      r#"{"version":3,"sources":["document.js"],"sourcesContent":["if (a;b;c) {\n  a; b; c;\n}"],"names":[],"mappings":"AAAA,IAAI,KAAK,GAET"}"#
    );
  }

  #[test]
  fn test_edge_case() {
    let line1 = "hello world\n";
    let mut source = ReplaceSource::new(OriginalSource::new(line1, "file.txt"));

    source.insert_static(0, "start1\n", None);
    source.replace_static(0, 0, "start2\n", None);
    source.replace_static(999, 10000, "end2", None);
    source.insert_static(888, "end1\n", None);
    source.replace_static(0, 999, "replaced!\n", Some("whole"));

    let result_text = source.source();
    let result_map = source
      .map(&ObjectPool::default(), &MapOptions::default())
      .unwrap();

    assert_eq!(
      result_text.into_string_lossy(),
      "start1\nstart2\nreplaced!\nend1\nend2"
    );
    assert_eq!(
      with_readable_mappings(&result_map),
      r#"
1:0 -> [file.txt] 1:0
2:0 -> [file.txt] 1:0
3:0 -> [file.txt] 1:0 (whole)"#
    );
  }

  #[test]
  fn replace_source_over_a_box_source() {
    let mut source = ReplaceSource::new(RawStringSource::from("boxed").boxed());
    source.replace_static(3, 5, "", None);
    assert_eq!(source.size(), 3);
    assert_eq!(source.source().into_string_lossy(), "box");
    assert_eq!(
      source.map(&ObjectPool::default(), &MapOptions::default()),
      None
    );
    let mut hasher = twox_hash::XxHash64::default();
    source.hash(&mut hasher);
    assert_eq!(format!("{:x}", hasher.finish()), "96abdb94c6fd5aba");
  }

  #[test]
  fn should_replace_correctly_with_unicode() {
    let content = r#"
"abc"; url(__PUBLIC_PATH__logo.png);
"ヒラギノ角ゴ"; url(__PUBLIC_PATH__logo.png);
"游ゴシック体"; url(__PUBLIC_PATH__logo.png);
"🤪"; url(__PUBLIC_PATH__logo.png);
"👨‍👩‍👧‍👧"; url(__PUBLIC_PATH__logo.png);
"#;
    let mut source = ReplaceSource::new(OriginalSource::new(content, "file.css").boxed());
    for mat in regex::Regex::new("__PUBLIC_PATH__")
      .unwrap()
      .find_iter(content)
    {
      source.replace_static(mat.start() as u32, mat.end() as u32, "../", None);
    }
    assert_eq!(
      source.source().into_string_lossy(),
      r#"
"abc"; url(../logo.png);
"ヒラギノ角ゴ"; url(../logo.png);
"游ゴシック体"; url(../logo.png);
"🤪"; url(../logo.png);
"👨‍👩‍👧‍👧"; url(../logo.png);
"#
    );
    assert_eq!(
      source
        .map(&ObjectPool::default(), &MapOptions::default())
        .unwrap()
        .to_json(),
      r#"{"version":3,"sources":["file.css"],"sourcesContent":["\n\"abc\"; url(__PUBLIC_PATH__logo.png);\n\"ヒラギノ角ゴ\"; url(__PUBLIC_PATH__logo.png);\n\"游ゴシック体\"; url(__PUBLIC_PATH__logo.png);\n\"🤪\"; url(__PUBLIC_PATH__logo.png);\n\"👨‍👩‍👧‍👧\"; url(__PUBLIC_PATH__logo.png);\n"],"names":[],"mappings":";AACA,OAAO,IAAI,GAAe;AAC1B,UAAU,IAAI,GAAe;AAC7B,UAAU,IAAI,GAAe;AAC7B,MAAM,IAAI,GAAe;AACzB,eAAe,IAAI,GAAe"}"#,
    );
  }

  #[test]
  fn replace_same_position_with_enforce() {
    // Enforce sort HarmonyExportExpressionDependency after PureExpressionDependency, to generate valid code
    let mut source = ReplaceSource::new(RawStringSource::from("export default foo;aaa").boxed());
    let mut source2 = source.clone();
    source.replace_static(18, 19, ");", None);
    source.replace_static(18, 19, "))", None);
    assert_eq!(
      source.source().into_string_lossy(),
      "export default foo);))aaa"
    );

    source2.replace_static_with_enforce(18, 19, ");", None, ReplacementEnforce::Post);
    source2.replace_static(18, 19, "))", None);
    assert_eq!(
      source2.source().into_string_lossy(),
      "export default foo)));aaa"
    );
  }

  #[test]
  fn test_debug_output() {
    let mut source = ReplaceSource::new(OriginalSource::new("hello", "file.txt").boxed());
    source.replace_static(0, 0, "println!(\"", None);
    source.replace_static(5, 5, "\")", None);
    assert_eq!(
      format!("{:?}", source),
      r#"{
  let mut source = ReplaceSource::new(
    OriginalSource::new(
      "hello",
      "file.txt",
    ).boxed()
  );
  source.replace(0, 0, "println!(\"", None);
  source.replace(5, 5, "\")", None);
  source.boxed()
}"#
    );
  }

  #[test]
  fn size_matches_generated_content_len() {
    let mut source = ReplaceSource::new(
      RawStringSource::from_static("import { jsx as _jsx, jsxs as _jsxs } from \"react/jsx-runtime\";\nimport React from 'react';\nimport Component__0 from './d0/f0.jsx';\n// import Component__1 from './d0/f1.jsx'\n// import Component__2 from './d0/f2.jsx'\n// import Component__3 from './d0/f3.jsx'\n// import Component__4 from './d0/f4.jsx'\n// import Component__5 from './d0/f5.jsx'\n// import Component__6 from './d0/f6.jsx'\n// import Component__7 from './d0/f7.jsx'\n// import Component__8 from './d0/f8.jsx'\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsxs(\"div\", {\n        children: [\n            /*#__PURE__*/ _jsx(Component__0, {}),\n            /*#__PURE__*/ _jsx(Component__1, {}),\n            /*#__PURE__*/ _jsx(Component__2, {}),\n            /*#__PURE__*/ _jsx(Component__3, {}),\n            /*#__PURE__*/ _jsx(Component__4, {}),\n            /*#__PURE__*/ _jsx(Component__5, {}),\n            /*#__PURE__*/ _jsx(Component__6, {}),\n            /*#__PURE__*/ _jsx(Component__7, {}),\n            /*#__PURE__*/ _jsx(Component__8, {})\n        ]\n    });\n}\nexport default Navbar;\n").boxed()
    );
    source.replace_static(0, 63, "", None);
    source.replace_static(64, 90, "", None);
    source.replace_static(91, 130, "", None);
    source.replace_static(
      544,
      549,
      "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsxs)",
      None,
    );
    source.replace_static(
      605,
      609,
      "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)",
      None,
    );
    source.replace_static(
      610,
      622,
      "_d0_f0_jsx__WEBPACK_IMPORTED_MODULE_2__[\"default\"]",
      None,
    );
    source.replace_static(
      655,
      659,
      "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)",
      None,
    );
    source.replace_static(
      705,
      709,
      "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)",
      None,
    );
    source.replace_static(
      755,
      759,
      "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)",
      None,
    );
    source.replace_static(
      805,
      809,
      "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)",
      None,
    );
    source.replace_static(
      855,
      859,
      "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)",
      None,
    );
    source.replace_static(
      905,
      909,
      "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)",
      None,
    );
    source.replace_static(
      955,
      959,
      "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)",
      None,
    );
    source.replace_static(
      1005,
      1009,
      "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)",
      None,
    );
    source.replace_static(
      1048,
      1063,
      "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (",
      None,
    );
    source.replace_static(1048, 1063, "", None);
    source.replace_static_with_enforce(1069, 1070, ");", None, ReplacementEnforce::Post);
    source.replace_static(10000000, 20000000, "// end line", None);

    assert_eq!(source.size(), source.source().into_string_lossy().len());
  }

  #[test]
  fn replace_source_hash_is_order_independent() {
    let mut source1 = ReplaceSource::new(RawStringSource::from_static("hello, world!").boxed());
    source1.replace_static(0, 5, "你好", None);
    source1.replace_static(6, 11, "世界", None);

    let mut source2 = ReplaceSource::new(RawStringSource::from_static("hello, world!").boxed());
    source2.replace_static(6, 11, "世界", None);
    source2.replace_static(0, 5, "你好", None);

    assert_eq!(source1.source(), source2.source());

    let mut hasher1 = FxHasher::default();
    source1.hash(&mut hasher1);
    let mut hasher2 = FxHasher::default();
    source2.hash(&mut hasher2);
    assert_eq!(hasher1.finish(), hasher2.finish());
  }

  #[test]
  fn test_replace_source_with_multi_unit_utf16() {
    let mut source = ReplaceSource::new(
      SourceMapSource::new(SourceMapSourceOptions {
          value: "var i18n = JSON.parse('{\"魑魅魍魉\":{\"en-US\":\"Evil spirits\",\"zh-CN\":\"魑魅魍魉\"}}');\nvar __webpack_exports___ = i18n[\"魑魅魍魉\"];\nexport { __webpack_exports___ as 魑魅魍魉 };\n",
          name: "main.js",
          source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"i18n.js\"],\"sourcesContent\":[\"var i18n = JSON.parse('{\\\"魑魅魍魉\\\":{\\\"en-US\\\":\\\"Evil spirits\\\",\\\"zh-CN\\\":\\\"魑魅魍魉\\\"}}');\\nvar __webpack_exports___ = i18n[\\\"魑魅魍魉\\\"];\\nexport { __webpack_exports___ as 魑魅魍魉 };\\n\"],\"names\":[\"i18n\",\"JSON\",\"__webpack_exports___\",\"魑魅魍魉\"],\"mappings\":\"AAAA,IAAIA,OAAOC,KAAK,KAAK,CAAC;AACtB,IAAIC,uBAAuBF,IAAI,CAAC,OAAO;AACvC,SAASE,wBAAwBC,IAAI,GAAG\"}").unwrap(),
          original_source: None,
          inner_source_map: None,
          remove_original_source: false,
        }).boxed()
      );
    source.replace_static(140, 188, "", None);

    assert_eq!(
      source.source().into_string_lossy(),
      "var i18n = JSON.parse('{\"魑魅魍魉\":{\"en-US\":\"Evil spirits\",\"zh-CN\":\"魑魅魍魉\"}}');\nvar __webpack_exports___ = i18n[\"魑魅魍魉\"];\n\n"
    );
    assert_eq!(source.map(&ObjectPool::default(), &MapOptions::default()).unwrap(), SourceMap::from_json(
      r#"{
          "version": 3,
          "sources": ["i18n.js"],
          "mappings": "AAAA,IAAIA,OAAOC,KAAK,KAAK,CAAC;AACtB,IAAIC,uBAAuBF,IAAI,CAAC,OAAO;AACC",
          "names": ["i18n", "JSON", "__webpack_exports___", "魑魅魍魉"],
          "sourcesContent": ["var i18n = JSON.parse('{\"魑魅魍魉\":{\"en-US\":\"Evil spirits\",\"zh-CN\":\"魑魅魍魉\"}}');\nvar __webpack_exports___ = i18n[\"魑魅魍魉\"];\nexport { __webpack_exports___ as 魑魅魍魉 };\n"]
        }"#
    ).unwrap());
  }

  #[test]
  fn test_replace_source_handle_remaining_replacements() {
    let mut source = ReplaceSource::new(OriginalSource::new("สวัสดี ชาวโลก!", "test.txt"));
    source.replace_static(0, 19, "hello, ", None);
    source.replace_static(19, 38, "world!", None);
    source.replace_static(100, 200, "\n", None);
    source.replace_static(200, 300, "你好，世界！\n", None);
    source.replace_static(300, 400, "こんにちは、世界！\n안녕하세요, 세계! \n", None);

    assert_eq!(
      source.source().into_string_lossy(),
      "hello, world!\n你好，世界！\nこんにちは、世界！\n안녕하세요, 세계! \n"
    );

    let mut chunks = vec![];
    let object_pool = ObjectPool::default();
    let handle = source.stream_chunks();
    handle.stream(
      &object_pool,
      &MapOptions::default(),
      &mut |chunk, mapping| {
        chunks.push((chunk.unwrap().as_str(), mapping));
      },
      &mut |_source_index, _source, _source_content| {},
      &mut |_name_index, _name| {},
    );

    assert_eq!(
      chunks,
      vec![
        (
          "hello, ",
          Mapping {
            generated_line: 1,
            generated_column: 0,
            original: Some(OriginalLocation {
              source_index: 0,
              original_line: 1,
              original_column: 0,
              name_index: None
            })
          }
        ),
        (
          "world!",
          Mapping {
            generated_line: 1,
            generated_column: 7,
            original: Some(OriginalLocation {
              source_index: 0,
              original_line: 1,
              original_column: 19,
              name_index: None
            })
          }
        ),
        (
          "\n",
          Mapping {
            generated_line: 1,
            generated_column: 13,
            original: None
          }
        ),
        (
          "你好，世界！\n",
          Mapping {
            generated_line: 2,
            generated_column: 0,
            original: None
          }
        ),
        (
          "こんにちは、世界！\n",
          Mapping {
            generated_line: 3,
            generated_column: 0,
            original: None
          }
        ),
        (
          "안녕하세요, 세계! \n",
          Mapping {
            generated_line: 4,
            generated_column: 0,
            original: None
          }
        )
      ]
    );
  }
}
