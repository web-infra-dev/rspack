use owo_colors::{OwoColorize, Stream};

/**
 * Dim the text if the stream supports color.
 */
pub fn dim(text: impl std::fmt::Display) -> String {
  text
    .if_supports_color(Stream::Stdout, |text| text.dimmed())
    .to_string()
}
