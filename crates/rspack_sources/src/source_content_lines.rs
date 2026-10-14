use std::borrow::Cow;

use crate::line_index::{IndexedLine, LineIndex};

#[cfg_attr(allocative, derive(allocative::Allocative))]
pub struct SourceContentLines<'source> {
  text: Cow<'source, str>,
  // Self-referential data structure: the index borrows from the text.
  index: LineIndex<'static>,
}

impl<'source> SourceContentLines<'source> {
  pub fn new(text: Cow<'source, str>) -> Self {
    // SAFETY: We extend the lifetime of the &str to 'static because the text is held by this struct,
    // and all &'static str references are only used within the lifetime of this struct.
    #[allow(unsafe_code)]
    let text_ref = unsafe { std::mem::transmute::<&str, &'static str>(text.as_ref()) };
    let index = LineIndex::new(text_ref);
    Self { text, index }
  }

  pub fn get(&self, line: usize) -> Option<IndexedLine<'_, '_>> {
    let _ = &self.text;
    self.index.get(line)
  }
}
