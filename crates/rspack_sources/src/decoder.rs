use std::slice::Iter;

use crate::{Mapping, OriginalLocation};

const COM: u8 = 0x40; // END_SEGMENT_BIT
const SEM: u8 = COM | 0x01; // NEXT_LINE
const ERR: u8 = COM | 0x02; // INVALID

const CONTINUATION_BIT: u8 = 0x20;
const DATA_MASK: u8 = 0x1f;

#[rustfmt::skip]
const B64: [u8; 256] = [
//  0    1    2    3    4    5    6    7    8    9    A    B    C    D    E    F    //
   ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR,  // 0
   ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR,  // 1
   ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR,  62, COM, ERR, ERR,  63,  // 2
    52,  53,  54,  55,  56,  57,  58,  59,  60,  61, ERR, SEM, ERR, ERR, ERR, ERR,  // 3
   ERR,   0,   1,   2,   3,   4,   5,   6,   7,   8,   9,  10,  11,  12,  13,  14,  // 4
    15,  16,  17,  18,  19,  20,  21,  22,  23,  24,  25, ERR, ERR, ERR, ERR, ERR,  // 5
   ERR,  26,  27,  28,  29,  30,  31,  32,  33,  34,  35,  36,  37,  38,  39,  40,  // 6
    41,  42,  43,  44,  45,  46,  47,  48,  49,  50,  51, ERR, ERR, ERR, ERR, ERR,  // 7
   ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR,  // 8
   ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR,  // 9
   ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR,  // A
   ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR,  // B
   ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR,  // C
   ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR,  // D
   ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR,  // E
   ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR,  // F
];

pub(crate) struct MappingsDecoder<'a> {
  mappings_iter: Iter<'a, u8>,

  current_data: [u32; 5],
  current_data_pos: usize,
  // current_value will include a sign bit at bit 0
  current_value: i64,
  current_value_pos: usize,
  generated_line: u32,
}

impl<'a> MappingsDecoder<'a> {
  pub fn new(mappings: &'a str) -> Self {
    Self {
      mappings_iter: mappings.as_bytes().iter(),
      current_data: [0u32, 0u32, 1u32, 0u32, 0u32],
      current_data_pos: 0,
      // current_value will include a sign bit at bit 0
      current_value: 0,
      current_value_pos: 0,
      generated_line: 1,
    }
  }
}

impl Iterator for MappingsDecoder<'_> {
  type Item = Mapping;

  fn next(&mut self) -> Option<Self::Item> {
    for c in &mut self.mappings_iter {
      let value = B64[*c as usize];
      if value == ERR {
        continue;
      }
      if (value & COM) != 0 {
        let mut mapping = Mapping {
          generated_line: self.generated_line,
          generated_column: self.current_data[0],
          original: None,
        };
        let current_data_pos = self.current_data_pos;
        self.current_data_pos = 0;
        if value == SEM {
          self.generated_line += 1;
          self.current_data[0] = 0;
        }
        match current_data_pos {
          1 => return Some(mapping),
          4 => {
            mapping.original = Some(OriginalLocation {
              source_index: self.current_data[1],
              original_line: self.current_data[2],
              original_column: self.current_data[3],
              name_index: None,
            });
            return Some(mapping);
          }
          5 => {
            mapping.original = Some(OriginalLocation {
              source_index: self.current_data[1],
              original_line: self.current_data[2],
              original_column: self.current_data[3],
              name_index: Some(self.current_data[4]),
            });
            return Some(mapping);
          }
          _ => (),
        };
      } else if (value & CONTINUATION_BIT) == 0 {
        // last sextet
        self.current_value |= (value as i64) << self.current_value_pos;
        let final_value = if (self.current_value & 1) != 0 {
          -(self.current_value >> 1)
        } else {
          self.current_value >> 1
        };
        if self.current_data_pos < 5 {
          self.current_data[self.current_data_pos] =
            (self.current_data[self.current_data_pos] as i64 + final_value) as u32;
        }
        self.current_data_pos += 1;
        self.current_value_pos = 0;
        self.current_value = 0;
      } else {
        self.current_value |= ((value & DATA_MASK) as i64) << self.current_value_pos;
        self.current_value_pos += 5;
      }
    }

    // end current segment
    let current_data_pos = self.current_data_pos;
    self.current_data_pos = 0;
    match current_data_pos {
      1 => {
        return Some(Mapping {
          generated_line: self.generated_line,
          generated_column: self.current_data[0],
          original: None,
        });
      }
      4 => {
        return Some(Mapping {
          generated_line: self.generated_line,
          generated_column: self.current_data[0],
          original: Some(OriginalLocation {
            source_index: self.current_data[1],
            original_line: self.current_data[2],
            original_column: self.current_data[3],
            name_index: None,
          }),
        });
      }
      5 => {
        return Some(Mapping {
          generated_line: self.generated_line,
          generated_column: self.current_data[0],
          original: Some(OriginalLocation {
            source_index: self.current_data[1],
            original_line: self.current_data[2],
            original_column: self.current_data[3],
            name_index: Some(self.current_data[4]),
          }),
        });
      }
      _ => (),
    }

    None
  }
}
