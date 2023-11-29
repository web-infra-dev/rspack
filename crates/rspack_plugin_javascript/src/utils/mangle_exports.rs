pub const START_LOWERCASE_ALPHABET_CODE: u32 = b'a' as u32;
pub const START_UPPERCASE_ALPHABET_CODE: u32 = b'A' as u32;
pub const DELTA_A_TO_Z: u32 = b'z' as u32 - START_LOWERCASE_ALPHABET_CODE + 1u32;
pub const NUMBER_OF_IDENTIFIER_START_CHARS: u32 = DELTA_A_TO_Z * 2u32 + 2u32; // a-z A-Z _ $
pub const NUMBER_OF_IDENTIFIER_CONTINUATION_CHARS: u32 = NUMBER_OF_IDENTIFIER_START_CHARS + 10u32; // a-z A-Z _ $ 0-9

pub fn number_to_identifier(mut n: u32) -> String {
  if n >= NUMBER_OF_IDENTIFIER_START_CHARS {
    // use multiple letters
    return number_to_identifier(n % NUMBER_OF_IDENTIFIER_START_CHARS)
      + number_to_identifier_continuation(n / NUMBER_OF_IDENTIFIER_START_CHARS).as_str();
  }

  // lower case
  if n < DELTA_A_TO_Z {
    return char::from_u32(START_LOWERCASE_ALPHABET_CODE + n)
      .expect("should convert successfully")
      .to_string();
  }

  n -= DELTA_A_TO_Z;

  // upper case
  if n < DELTA_A_TO_Z {
    return char::from_u32(START_UPPERCASE_ALPHABET_CODE + n)
      .expect("should convert successfully")
      .to_string();
  }

  if n == DELTA_A_TO_Z {
    '_'.to_string()
  } else {
    '$'.to_string()
  }
}

pub fn number_to_identifier_continuation(mut n: u32) -> String {
  if n >= NUMBER_OF_IDENTIFIER_CONTINUATION_CHARS {
    // use multiple letters
    return number_to_identifier_continuation(n % NUMBER_OF_IDENTIFIER_CONTINUATION_CHARS)
      + number_to_identifier_continuation(n / NUMBER_OF_IDENTIFIER_CONTINUATION_CHARS).as_str();
  }

  // lower case
  if n < DELTA_A_TO_Z {
    return char::from_u32(START_LOWERCASE_ALPHABET_CODE + n)
      .expect("should convert successfully")
      .to_string();
  }
  n -= DELTA_A_TO_Z;

  // upper case
  if n < DELTA_A_TO_Z {
    return char::from_u32(START_UPPERCASE_ALPHABET_CODE + n)
      .expect("should convert successfully")
      .to_string();
  }

  n -= DELTA_A_TO_Z;

  // numbers
  if n < 10 {
    return char::from_u32(n)
      .expect("should convert successfully")
      .to_string();
  }

  if n == 10 {
    '_'.to_string()
  } else {
    '$'.to_string()
  }
}
