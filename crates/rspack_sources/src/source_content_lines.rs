use std::borrow::Cow;

use crate::{helpers::split_into_lines, with_utf16::WithUtf16};

#[cfg_attr(allocative, derive(allocative::Allocative))]
pub struct SourceContentLines<'source> {
  text: Cow<'source, str>,
  // Self-referential data structure: lines borrow from the text.
  lines: Vec<WithUtf16<'static>>,
}

impl<'source> SourceContentLines<'source> {
  pub fn new(text: Cow<'source, str>) -> Self {
    // SAFETY: We extend the lifetime of the &str to 'static because the text is held by this struct,
    // and all &'static str references are only used within the lifetime of this struct.
    #[allow(unsafe_code)]
    let text_ref = unsafe { std::mem::transmute::<&str, &'static str>(text.as_ref()) };
    let lines = split_into_lines(text_ref)
      .map(WithUtf16::new)
      .collect::<Vec<_>>();
    Self { text, lines }
  }

  pub fn get(&self, line: usize) -> Option<&WithUtf16<'_>> {
    let _ = &self.text;
    self.lines.get(line)
  }
}
