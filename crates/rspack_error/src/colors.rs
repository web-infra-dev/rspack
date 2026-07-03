use owo_colors::{OwoColorize, Stream::Stdout};

/// Dim the string if the stream supports color.
#[inline]
pub fn dim_str(text: &str) -> String {
  text.if_supports_color(Stdout, |t| t.dimmed()).to_string()
}

/// Color the string red if the stream supports color.
#[inline]
pub fn red_str(text: &str) -> String {
  text.if_supports_color(Stdout, |t| t.red()).to_string()
}

/// Color the string yellow if the stream supports color.
#[inline]
pub fn yellow_str(text: &str) -> String {
  text.if_supports_color(Stdout, |t| t.yellow()).to_string()
}

/// Color the string cyan if the stream supports color.
#[inline]
pub fn cyan_str(text: &str) -> String {
  text.if_supports_color(Stdout, |t| t.cyan()).to_string()
}
