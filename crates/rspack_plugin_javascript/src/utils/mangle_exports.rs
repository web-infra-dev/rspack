const START_LOWERCASE_ALPHABET_CODE: u32 = b'a';
const DELTA_A_TO_Z: u32 = b'z' - START_LOWERCASE_ALPHABET_CODE + 1u32;
const NUMBER_OF_IDENTIFIER_START_CHARS: u32 = DELTA_A_TO_Z * 2u32 + 2u32; // a-z A-Z _ $
const NUMBER_OF_IDENTIFIER_CONTINUATION_CHARS: u32 = NUMBER_OF_IDENTIFIER_START_CHARS + 10; // a-z A-Z _ $ 0-9

fn number_to_identifier(mut n: u32) -> char {
  if (n >= NUMBER_OF_IDENTIFIER_START_CHARS) {
    // use multiple letters
    return char::from_u32(
      numberToIdentifier(n % NUMBER_OF_IDENTIFIER_START_CHARS) as u32
        + numberToIdentifierContinuation(n / NUMBER_OF_IDENTIFIER_START_CHARS) as u32,
    );
  }

  // lower case
  if n < DELTA_A_TO_Z {
    return char::from_u32(START_LOWERCASE_ALPHABET_CODE + n);
  }

  n -= DELTA_A_TO_Z;

  // upper case
  if n < DELTA_A_TO_Z {
    return char::from_u32(START_UPPERCASE_ALPHABET_CODE + n);
  }

  if n == DELTA_A_TO_Z {
    return '_';
  };
  return '$';
}

pub fn number_to_identifier_continuation(mut n: u32) -> char {
  if n >= NUMBER_OF_IDENTIFIER_CONTINUATION_CHARS {
    // use multiple letters
    return (numberToIdentifierContinuation(n % NUMBER_OF_IDENTIFIER_CONTINUATION_CHARS)
      + numberToIdentifierContinuation(n / NUMBER_OF_IDENTIFIER_CONTINUATION_CHARS));
  }

  // lower case
  if n < DELTA_A_TO_Z {
    return char::from_u32(START_LOWERCASE_ALPHABET_CODE + n);
  }
  n -= DELTA_A_TO_Z;

  // upper case
  if (n < DELTA_A_TO_Z) {
    return char::from_u32(START_UPPERCASE_ALPHABET_CODE + n);
  }

  n -= DELTA_A_TO_Z;

  // numbers
  if n < 10 {
    return char::from_u32(n);
  }

  if n == 10 {
    return '_';
  };
  return '$';
}
