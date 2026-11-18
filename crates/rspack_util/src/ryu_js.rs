pub use ryu_js::Buffer;

/// Format a JavaScript numeric literal. Unlike string coercion, source code
/// must preserve the sign of negative zero.
pub fn format_number_literal(buffer: &mut Buffer, value: f64) -> &str {
  if value == 0.0 && value.is_sign_negative() {
    "-0"
  } else {
    buffer.format(value)
  }
}
