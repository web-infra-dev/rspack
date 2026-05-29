use crate::Mapping;

const B64_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

#[inline(always)]
pub fn encode_vlq(out: &mut Vec<u8>, a: u32, b: u32) {
  if a == b {
    out.push(b'A');
    return;
  }

  let mut num = if a >= b {
    (a - b) << 1
  } else {
    ((b - a) << 1) + 1
  };

  if num < 0b100000 {
    out.push(B64_CHARS[num as usize]);
    return;
  }

  loop {
    let mut digit = num & 0b11111;
    num >>= 5;
    if num > 0 {
      digit |= 1 << 5;
    }
    out.push(B64_CHARS[digit as usize]);
    if num == 0 {
      break;
    }
  }
}

pub(crate) enum MappingsEncoder {
  Full(FullMappingsEncoder),
  LinesOnly(LinesOnlyMappingsEncoder),
}

impl MappingsEncoder {
  #[inline(always)]
  pub fn encode(&mut self, mapping: &Mapping) {
    match self {
      MappingsEncoder::Full(enc) => enc.encode(mapping),
      MappingsEncoder::LinesOnly(enc) => enc.encode(mapping),
    }
  }

  #[inline]
  pub fn drain(&mut self) -> String {
    match self {
      MappingsEncoder::Full(enc) => enc.drain(),
      MappingsEncoder::LinesOnly(enc) => enc.drain(),
    }
  }
}

pub fn create_encoder(columns: bool) -> MappingsEncoder {
  if columns {
    MappingsEncoder::Full(FullMappingsEncoder::new())
  } else {
    MappingsEncoder::LinesOnly(LinesOnlyMappingsEncoder::new())
  }
}

pub(crate) struct FullMappingsEncoder {
  current_line: u32,
  current_column: u32,
  current_original_line: u32,
  current_original_column: u32,
  current_source_index: u32,
  current_name_index: u32,
  active_mapping: bool,
  active_name: bool,
  initial: bool,
  mappings: Vec<u8>,
}

impl FullMappingsEncoder {
  pub fn new() -> Self {
    Self {
      current_line: 1,
      current_column: 0,
      current_original_line: 1,
      current_original_column: 0,
      current_source_index: 0,
      current_name_index: 0,
      active_mapping: false,
      active_name: false,
      initial: true,
      mappings: Default::default(),
    }
  }
}

impl FullMappingsEncoder {
  #[inline(always)]
  fn encode(&mut self, mapping: &Mapping) {
    if self.active_mapping && self.current_line == mapping.generated_line {
      // A mapping is still active
      if mapping.original.as_ref().is_some_and(|original| {
        original.source_index == self.current_source_index
          && original.original_line == self.current_original_line
          && original.original_column == self.current_original_column
          && !self.active_name
          && original.name_index.is_none()
      }) {
        // avoid repeating the same original mapping
        return;
      }
    } else {
      // No mapping is active
      if mapping.original.is_none() {
        // avoid writing unnecessary generated mappings
        return;
      }
    }

    if self.current_line < mapping.generated_line {
      let count = (mapping.generated_line - self.current_line) as usize;
      self.mappings.extend(std::iter::repeat_n(b';', count));
      self.current_line = mapping.generated_line;
      self.current_column = 0;
      self.initial = false;
    } else if self.initial {
      self.initial = false;
    } else {
      self.mappings.push(b',');
    }

    encode_vlq(
      &mut self.mappings,
      mapping.generated_column,
      self.current_column,
    );
    self.current_column = mapping.generated_column;
    if let Some(original) = &mapping.original {
      self.active_mapping = true;
      if original.source_index == self.current_source_index {
        self.mappings.push(b'A');
      } else {
        encode_vlq(
          &mut self.mappings,
          original.source_index,
          self.current_source_index,
        );
        self.current_source_index = original.source_index;
      }
      if original.original_line == self.current_original_line {
        self.mappings.push(b'A');
      } else {
        encode_vlq(
          &mut self.mappings,
          original.original_line,
          self.current_original_line,
        );
        self.current_original_line = original.original_line;
      }
      if original.original_column == self.current_original_column {
        self.mappings.push(b'A');
      } else {
        encode_vlq(
          &mut self.mappings,
          original.original_column,
          self.current_original_column,
        );
        self.current_original_column = original.original_column;
      }
      if let Some(name_index) = original.name_index {
        encode_vlq(&mut self.mappings, name_index, self.current_name_index);
        self.current_name_index = name_index;
        self.active_name = true;
      } else {
        self.active_name = false;
      }
    } else {
      self.active_mapping = false;
    }
  }

  #[allow(unsafe_code)]
  #[inline]
  fn drain(&mut self) -> String {
    unsafe {
      // SAFETY: The `mappings` field in the source map consists solely of ASCII characters.
      String::from_utf8_unchecked(std::mem::take(&mut self.mappings))
    }
  }
}

pub(crate) struct LinesOnlyMappingsEncoder {
  last_written_line: u32,
  current_line: u32,
  current_source_index: u32,
  current_original_line: u32,
  mappings: Vec<u8>,
}

impl LinesOnlyMappingsEncoder {
  pub fn new() -> Self {
    Self {
      last_written_line: 0,
      current_line: 1,
      current_source_index: 0,
      current_original_line: 1,
      mappings: Default::default(),
    }
  }
}

impl LinesOnlyMappingsEncoder {
  #[inline(always)]
  fn encode(&mut self, mapping: &Mapping) {
    if let Some(original) = &mapping.original {
      if self.last_written_line == mapping.generated_line {
        // avoid writing multiple original mappings per line
        return;
      }
      self.last_written_line = mapping.generated_line;

      let line_delta = mapping.generated_line - self.current_line;
      if line_delta > 0 {
        self
          .mappings
          .extend(std::iter::repeat_n(b';', line_delta as usize));
      }

      self.current_line = mapping.generated_line;

      if original.source_index == self.current_source_index {
        if original.original_line == self.current_original_line + 1 {
          self.current_original_line = original.original_line;
          self.mappings.extend(b"AACA");
        } else {
          self.mappings.extend(b"AA");
          encode_vlq(
            &mut self.mappings,
            original.original_line,
            self.current_original_line,
          );
          self.current_original_line = original.original_line;
          self.mappings.push(b'A');
        }
      } else {
        self.mappings.extend(b"A");
        encode_vlq(
          &mut self.mappings,
          original.source_index,
          self.current_source_index,
        );
        self.current_source_index = original.source_index;
        encode_vlq(
          &mut self.mappings,
          original.original_line,
          self.current_original_line,
        );
        self.current_original_line = original.original_line;
        self.mappings.push(b'A');
      }
    }
  }

  #[allow(unsafe_code)]
  #[inline]
  fn drain(&mut self) -> String {
    unsafe {
      // SAFETY: The `mappings` field in the source map consists solely of ASCII characters.
      String::from_utf8_unchecked(std::mem::take(&mut self.mappings))
    }
  }
}
