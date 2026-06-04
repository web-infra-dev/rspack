use std::sync::Arc;

use crate::{helpers::split_into_lines, object_pool::ObjectPool, with_utf16::WithUtf16};

pub struct SourceContentLines<'object_pool> {
  text: Arc<str>,
  // Self-referential data structure: lines borrow from the text.
  lines: Vec<WithUtf16<'object_pool, 'static>>,
}

impl<'object_pool> SourceContentLines<'object_pool> {
  pub fn new(object_pool: &'object_pool ObjectPool, text: Arc<str>) -> Self {
    // SAFETY: We extend the lifetime of the &str to 'static because the Arc<str> is owned by this struct,
    // and all &'static str references are only used within the lifetime of this struct.
    #[allow(unsafe_code)]
    let text_ref = unsafe { std::mem::transmute::<&str, &'static str>(text.as_ref()) };
    let lines = split_into_lines(text_ref)
      .map(|line| WithUtf16::new(object_pool, line))
      .collect::<Vec<_>>();
    Self { text, lines }
  }

  pub fn get(&self, line: usize) -> Option<&WithUtf16<'object_pool, '_>> {
    let _ = &self.text;
    self.lines.get(line)
  }
}
