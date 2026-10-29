//! Color math (`tree/color.js`).

use crate::{
  css::CssCtx,
  num::{self, js_round},
  tree::Color,
};

pub fn clamp(v: f64, max: f64) -> f64 {
  v.max(0.0).min(max)
}

/// `Math.min(Math.max(v, 0), max)` with JS NaN semantics.
fn clamp_js(v: f64, max: f64) -> f64 {
  if v.is_nan() {
    return f64::NAN;
  }
  clamp(v, max)
}

pub fn to_hex(v: &[f64]) -> String {
  let mut s = String::with_capacity(1 + v.len() * 2);
  write_hex(&mut s, v);
  s
}

pub fn write_hex(s: &mut String, v: &[f64]) {
  s.push('#');
  for &c in v {
    let c = clamp_js(js_round(c), 255.0);
    if c.is_nan() {
      s.push_str("NaN");
      continue;
    }
    let c = c as u32;
    const HEX: &[u8; 16] = b"0123456789abcdef";
    s.push(HEX[(c >> 4) as usize & 15] as char);
    s.push(HEX[(c & 15) as usize] as char);
  }
}

pub struct Hsl {
  pub h: f64,
  pub s: f64,
  pub l: f64,
  pub a: f64,
}

pub struct Hsv {
  pub h: f64,
  pub s: f64,
  pub v: f64,
  pub a: f64,
}

fn js_max3(a: f64, b: f64, c: f64) -> f64 {
  if a.is_nan() || b.is_nan() || c.is_nan() {
    return f64::NAN;
  }
  a.max(b).max(c)
}
fn js_min3(a: f64, b: f64, c: f64) -> f64 {
  if a.is_nan() || b.is_nan() || c.is_nan() {
    return f64::NAN;
  }
  a.min(b).min(c)
}

pub fn to_hsl(c: &Color<'_>) -> Hsl {
  let r = c.rgb[0] / 255.0;
  let g = c.rgb[1] / 255.0;
  let b = c.rgb[2] / 255.0;
  let a = c.alpha;
  let max = js_max3(r, g, b);
  let min = js_min3(r, g, b);
  let l = (max + min) / 2.0;
  let d = max - min;
  let (h, s);
  if max == min {
    h = 0.0;
    s = 0.0;
  } else {
    s = if l > 0.5 {
      d / (2.0 - max - min)
    } else {
      d / (max + min)
    };
    let hh = if max == r {
      (g - b) / d + if g < b { 6.0 } else { 0.0 }
    } else if max == g {
      (b - r) / d + 2.0
    } else if max == b {
      (r - g) / d + 4.0
    } else {
      f64::NAN
    };
    h = hh / 6.0;
  }
  Hsl {
    h: h * 360.0,
    s,
    l,
    a,
  }
}

pub fn to_hsv(c: &Color<'_>) -> Hsv {
  let r = c.rgb[0] / 255.0;
  let g = c.rgb[1] / 255.0;
  let b = c.rgb[2] / 255.0;
  let a = c.alpha;
  let max = js_max3(r, g, b);
  let min = js_min3(r, g, b);
  let v = max;
  let d = max - min;
  let s = if max == 0.0 { 0.0 } else { d / max };
  let h = if max == min {
    0.0
  } else {
    let hh = if max == r {
      (g - b) / d + if g < b { 6.0 } else { 0.0 }
    } else if max == g {
      (b - r) / d + 2.0
    } else {
      (r - g) / d + 4.0
    };
    hh / 6.0
  };
  Hsv {
    h: h * 360.0,
    s,
    v,
    a,
  }
}

pub fn luma(c: &Color<'_>) -> f64 {
  let f = |x: f64| {
    let x = x / 255.0;
    if x <= 0.03928 {
      x / 12.92
    } else {
      ((x + 0.055) / 1.055).powf(2.4)
    }
  };
  0.2126 * f(c.rgb[0]) + 0.7152 * f(c.rgb[1]) + 0.0722 * f(c.rgb[2])
}

pub fn to_argb(c: &Color<'_>) -> String {
  to_hex(&[c.alpha * 255.0, c.rgb[0], c.rgb[1], c.rgb[2]])
}

/// `Color.toCSS(context, doNotCompress)`
pub fn to_css(c: &Color<'_>, ctx: &CssCtx, do_not_compress: bool) -> String {
  let mut s = String::new();
  write_css(&mut s, c, ctx, do_not_compress);
  s
}

/// Appends `Color.toCSS` to `out`.
pub fn write_css(out: &mut String, c: &Color<'_>, ctx: &CssCtx, do_not_compress: bool) {
  let compress = ctx.compress && !do_not_compress;
  let alpha = ctx.fround(c.alpha);
  let mut color_function: Option<&str> = None;
  if let Some(v) = c.value.get().filter(|v| !v.is_empty()) {
    if v.starts_with("rgb") {
      if alpha < 1.0 {
        color_function = Some("rgba");
      }
    } else if v.starts_with("hsl") {
      color_function = Some(if alpha < 1.0 { "hsla" } else { "hsl" });
    } else {
      out.push_str(v);
      return;
    }
  } else if alpha < 1.0 {
    color_function = Some("rgba");
  }
  let sep = if compress { "," } else { ", " };
  match color_function {
    Some("rgba") => {
      out.push_str("rgba(");
      for &x in &c.rgb {
        num::write_js(out, clamp_js(js_round(x), 255.0));
        out.push_str(sep);
      }
      num::write_js(out, clamp_js(alpha, 1.0));
      out.push(')');
      return;
    }
    Some(f) => {
      let hsl = to_hsl(c);
      out.push_str(f);
      out.push('(');
      num::write_js(out, ctx.fround(hsl.h));
      out.push_str(sep);
      num::write_js(out, ctx.fround(hsl.s * 100.0));
      out.push('%');
      out.push_str(sep);
      num::write_js(out, ctx.fround(hsl.l * 100.0));
      out.push('%');
      if f == "hsla" {
        out.push_str(sep);
        num::write_js(out, clamp_js(alpha, 1.0));
      }
      out.push(')');
      return;
    }
    None => {}
  }
  let start = out.len();
  write_hex(out, &c.rgb);
  if compress {
    let b = &out.as_bytes()[start..];
    if b.len() == 7 && b[1] == b[2] && b[3] == b[4] && b[5] == b[6] {
      let short = [b'#', b[1], b[3], b[5]];
      out.truncate(start);
      out.push_str(std::str::from_utf8(&short).unwrap_or_default());
    }
  }
}
