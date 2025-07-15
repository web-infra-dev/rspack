pub use lightningcss::*;

pub fn monomorphized(black_box: fn(*const ())) {
  black_box(<lightningcss::stylesheet::StyleSheet<'_, '_>>::minify as *const ());
  black_box(<lightningcss::properties::Property as core::clone::Clone>::clone as *const ());
  black_box(<lightningcss::rules::CssRule as core::clone::Clone>::clone as *const ());
  black_box(<lightningcss::rules::CssRuleList as lightningcss::traits::ToCss>::to_css::<String> as *const ());
  black_box(<lightningcss::rules::font_face::FontFaceRule as lightningcss::traits::ToCss>::to_css::<String> as *const ());
  black_box(<lightningcss::properties::masking::ClipPath as core::clone::Clone>::clone as *const ());
  black_box(<lightningcss::properties::Property as core::cmp::PartialEq>::eq as *const ());
  black_box(<lightningcss::properties::masking::ClipPath as core::cmp::PartialEq>::eq as *const ());
  black_box(<lightningcss::properties::PropertyId as core::clone::Clone>::clone as *const ());
}
