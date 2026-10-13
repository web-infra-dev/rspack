//! Bounded buffering for non-overlapping Aho-Corasick searches over byte streams.

use std::io::{self, Write};

use aho_corasick::{AhoCorasick, Input, Match};

/// Preserve leftmost match selection across arbitrary write boundaries. A suffix
/// at least as long as the longest pattern stays pending until more input arrives.
pub struct StreamingMatcher<'a, F> {
  matcher: &'a AhoCorasick,
  on_match: F,
  buffer: Vec<u8>,
  capacity: usize,
  offset: usize,
  start: usize,
  last_match_end: Option<usize>,
}

impl<'a, F: FnMut(Match)> StreamingMatcher<'a, F> {
  pub fn new(matcher: &'a AhoCorasick, on_match: F) -> Self {
    let capacity = (64 * 1024).max(matcher.max_pattern_len().saturating_mul(2) + 1);
    Self {
      matcher,
      on_match,
      buffer: Vec::with_capacity(capacity),
      capacity,
      offset: 0,
      start: 0,
      last_match_end: None,
    }
  }

  /// Resolve pending matches at the end of the input.
  pub fn finish(mut self) {
    self.scan(true);
  }

  fn scan(&mut self, final_chunk: bool) {
    let limit = if final_chunk {
      self.buffer.len() + 1
    } else {
      self
        .buffer
        .len()
        .saturating_sub(self.matcher.max_pattern_len().max(1))
    };
    for found in self
      .matcher
      .find_iter(Input::new(&self.buffer).span(self.start - self.offset..self.buffer.len()))
    {
      if found.start() >= limit {
        break;
      }
      let end = self.offset + found.end();
      // Match the iterator's suppression of an empty match abutting the
      // previous match, including when the two appear in different windows.
      if found.is_empty() && self.last_match_end == Some(end) {
        continue;
      }
      (self.on_match)(Match::new(
        found.pattern(),
        self.offset + found.start()..end,
      ));
      self.last_match_end = Some(end);
      self.start = end;
    }
    let consumed = limit.max(self.start - self.offset).min(self.buffer.len());
    self.buffer.drain(..consumed);
    self.offset += consumed;
    self.start = self.start.max(self.offset);
  }
}

impl<F: FnMut(Match)> Write for StreamingMatcher<'_, F> {
  fn write(&mut self, mut bytes: &[u8]) -> io::Result<usize> {
    let len = bytes.len();
    while !bytes.is_empty() {
      let count = bytes.len().min(self.capacity - self.buffer.len());
      self.buffer.extend_from_slice(&bytes[..count]);
      bytes = &bytes[count..];
      if self.buffer.len() == self.capacity {
        self.scan(false);
      }
    }
    Ok(len)
  }

  fn flush(&mut self) -> io::Result<()> {
    Ok(())
  }
}
