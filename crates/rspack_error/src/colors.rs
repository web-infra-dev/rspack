use std::fmt::Display;

use owo_colors::{OwoColorize, Stream::Stdout};

/// Dim the string if the stream supports color.
#[inline]
pub fn dim_str(text: &str) -> String {
  text.if_supports_color(Stdout, |t| t.dimmed()).to_string()
}

/// Dim the text if the stream supports color.
#[inline]
pub fn dim<T>(text: &T) -> impl Display + '_
where
  T: Display + OwoColorize,
{
  text.if_supports_color(Stdout, |t| t.dimmed())
}

/// Color the string red if the stream supports color.
#[inline]
pub fn red_str(text: &str) -> String {
  text.if_supports_color(Stdout, |t| t.red()).to_string()
}

/// Color the text red if the stream supports color.
#[inline]
pub fn red<T>(text: &T) -> impl Display + '_
where
  T: Display + OwoColorize,
{
  text.if_supports_color(Stdout, |t| t.red())
}

/// Color the string yellow if the stream supports color.
#[inline]
pub fn yellow_str(text: &str) -> String {
  text.if_supports_color(Stdout, |t| t.yellow()).to_string()
}

/// Color the text yellow if the stream supports color.
#[inline]
pub fn yellow<T>(text: &T) -> impl Display + '_
where
  T: Display + OwoColorize,
{
  text.if_supports_color(Stdout, |t| t.yellow())
}

/// Color the string cyan if the stream supports color.
#[inline]
pub fn cyan_str(text: &str) -> String {
  text.if_supports_color(Stdout, |t| t.cyan()).to_string()
}

/// Color the text cyan if the stream supports color.
#[inline]
pub fn cyan<T>(text: &T) -> impl Display + '_
where
  T: Display + OwoColorize,
{
  text.if_supports_color(Stdout, |t| t.cyan())
}
