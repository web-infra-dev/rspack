use std::fmt::Display;

use owo_colors::{OwoColorize, Stream::Stdout};

/**
 * Dim the text if the stream supports color.
 */

#[inline]
pub fn dim<T>(text: &T) -> impl Display + '_
where
  T: Display + OwoColorize + ?Sized,
{
  text.if_supports_color(Stdout, |t| t.dimmed())
}

#[inline]
pub fn red<T>(text: &T) -> impl Display + '_
where
  T: Display + OwoColorize + ?Sized,
{
  text.if_supports_color(Stdout, |t| t.red())
}

#[inline]
pub fn yellow<T>(text: &T) -> impl Display + '_
where
  T: Display + OwoColorize + ?Sized,
{
  text.if_supports_color(Stdout, |t| t.yellow())
}

#[inline]
pub fn cyan<T>(text: &T) -> impl Display + '_
where
  T: Display + OwoColorize + ?Sized,
{
  text.if_supports_color(Stdout, |t| t.cyan())
}
