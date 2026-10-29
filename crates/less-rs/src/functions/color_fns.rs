//! Color functions (`functions/color.js`, `functions/color-blending.js`).

use super::{Registry, builtins::undefined_value};
use crate::{
  color::{self, Hsl, to_hsl, to_hsv},
  error::{LResult, LessError},
  eval::EvalCtx,
  syntax::parser::{color_from_hex, color_from_keyword},
  tree::*,
};

type R<'a> = LResult<Option<Node<'a>>>;

pub(super) fn register(r: &mut Registry) {
  r.add("rgb", rgb);
  r.add("rgba", rgba);
  r.add("hsl", hsl);
  r.add("hsla", hsla_fn);
  r.add("hsv", hsv);
  r.add("hsva", hsva_fn);
  r.add("hue", |c, _, a| {
    Ok(Some(c.a.dimension(to_hsl(col(a, 0)?).h, &EMPTY_UNIT)))
  });
  r.add("saturation", |c, _, a| {
    Ok(Some(c.a.dimension_str(to_hsl(col(a, 0)?).s * 100.0, "%")))
  });
  r.add("lightness", |c, _, a| {
    Ok(Some(c.a.dimension_str(to_hsl(col(a, 0)?).l * 100.0, "%")))
  });
  r.add("hsvhue", |c, _, a| {
    Ok(Some(c.a.dimension(to_hsv(col(a, 0)?).h, &EMPTY_UNIT)))
  });
  r.add("hsvsaturation", |c, _, a| {
    Ok(Some(c.a.dimension_str(to_hsv(col(a, 0)?).s * 100.0, "%")))
  });
  r.add("hsvvalue", |c, _, a| {
    Ok(Some(c.a.dimension_str(to_hsv(col(a, 0)?).v * 100.0, "%")))
  });
  r.add("red", |c, _, a| {
    Ok(Some(c.a.dimension(rgb_of(a, 0)?[0], &EMPTY_UNIT)))
  });
  r.add("green", |c, _, a| {
    Ok(Some(c.a.dimension(rgb_of(a, 0)?[1], &EMPTY_UNIT)))
  });
  r.add("blue", |c, _, a| {
    Ok(Some(c.a.dimension(rgb_of(a, 0)?[2], &EMPTY_UNIT)))
  });
  r.add("alpha", |c, _, a| {
    Ok(Some(c.a.dimension(to_hsl(col(a, 0)?).a, &EMPTY_UNIT)))
  });
  r.add("luma", |c, _, a| {
    let x = col(a, 0)?;
    Ok(Some(
      c.a.dimension_str(color::luma(x) * x.alpha * 100.0, "%"),
    ))
  });
  r.add("luminance", |c, _, a| {
    let x = col(a, 0)?;
    let l = (0.2126 * x.rgb[0] / 255.0) + (0.7152 * x.rgb[1] / 255.0) + (0.0722 * x.rgb[2] / 255.0);
    Ok(Some(c.a.dimension_str(l * x.alpha * 100.0, "%")))
  });
  r.add("saturate", |c, _, a| adjust(c, a, Adjust::S, 1.0, true));
  r.add("desaturate", |c, _, a| adjust(c, a, Adjust::S, -1.0, false));
  r.add("lighten", |c, _, a| adjust(c, a, Adjust::L, 1.0, false));
  r.add("darken", |c, _, a| adjust(c, a, Adjust::L, -1.0, false));
  r.add("fadein", |c, _, a| adjust(c, a, Adjust::A, 1.0, false));
  r.add("fadeout", |c, _, a| adjust(c, a, Adjust::A, -1.0, false));
  r.add("fade", fade);
  r.add("spin", spin);
  r.add("mix", mix);
  r.add("greyscale", |c, _, a| {
    let x = col(a, 0)?;
    let mut h = to_hsl(x);
    h.s -= 1.0; // desaturate(color, 100)
    h.s = clamp01(h.s);
    Ok(hsla_from(c, Node::Color(x), h))
  });
  r.add("contrast", contrast);
  r.add("argb", |c, _, a| {
    let x = col(a, 0)?;
    Ok(Some(c.a.anonymous(c.str(&color::to_argb(x)), Pos::NONE)))
  });
  r.add("color", color_fn);
  r.add("tint", |c, _, a| {
    let white = c.a.color([255.0, 255.0, 255.0], 1.0, Some("rgb"));
    mix_impl(c, Node::Color(white), a.first().copied(), a.get(1).copied())
  });
  r.add("shade", |c, _, a| {
    let black = c.a.color([0.0, 0.0, 0.0], 1.0, Some("rgb"));
    mix_impl(c, Node::Color(black), a.first().copied(), a.get(1).copied())
  });
  // color-blending.js
  r.add("multiply", |c, _, a| blend(c, a, |cb, cs| cb * cs));
  r.add("screen", |c, _, a| blend(c, a, screen));
  r.add("overlay", |c, _, a| blend(c, a, overlay));
  r.add("softlight", |c, _, a| {
    blend(c, a, |cb, cs| {
      let mut d = 1.0;
      let mut e = cb;
      if cs > 0.5 {
        e = 1.0;
        d = if cb > 0.25 {
          cb.sqrt()
        } else {
          ((16.0 * cb - 12.0) * cb + 4.0) * cb
        };
      }
      cb - (1.0 - 2.0 * cs) * e * (d - cb)
    })
  });
  r.add("hardlight", |c, _, a| blend(c, a, |cb, cs| overlay(cs, cb)));
  r.add("difference", |c, _, a| {
    blend(c, a, |cb, cs| (cb - cs).abs())
  });
  r.add("exclusion", |c, _, a| {
    blend(c, a, |cb, cs| cb + cs - 2.0 * cb * cs)
  });
  r.add("average", |c, _, a| blend(c, a, |cb, cs| (cb + cs) / 2.0));
  r.add("negation", |c, _, a| {
    blend(c, a, |cb, cs| 1.0 - (cb + cs - 1.0).abs())
  });
}

fn screen(cb: f64, cs: f64) -> f64 {
  cb + cs - cb * cs
}
fn overlay(cb: f64, cs: f64) -> f64 {
  let cb = cb * 2.0;
  if cb <= 1.0 {
    cb * cs
  } else {
    screen(cb - 1.0, cs)
  }
}

fn clamp01(v: f64) -> f64 {
  if v.is_nan() {
    return f64::NAN;
  }
  v.max(0.0).min(1.0)
}

/// `toHSL(color)`: requires a Color.
fn col<'a>(a: &[Node<'a>], i: usize) -> LResult<&'a Color<'a>> {
  match a.get(i) {
    Some(Node::Color(c)) => Ok(c),
    Some(_) => Err(LessError::native("Argument cannot be evaluated to a color")),
    None => Err(undefined_value("toHSL")),
  }
}

fn rgb_of<'a>(a: &[Node<'a>], i: usize) -> LResult<[f64; 3]> {
  match a.get(i) {
    Some(Node::Color(c)) => Ok(c.rgb),
    Some(_) => Err(undefined_value("0")),
    None => Err(undefined_value("rgb")),
  }
}

/// `number(n)` in color.js
fn number(n: Option<Node<'_>>) -> LResult<f64> {
  match n {
    Some(Node::Dimension(d)) => {
      let pct = is_percent(d.unit);
      Ok(if pct { d.value / 100.0 } else { d.value })
    }
    _ => Err(LessError::new(
      "Argument",
      "color functions take numbers as parameters",
    )),
  }
}

/// Either a node or a plain JS number (internal calls pass numbers).
#[derive(Copy, Clone)]
enum NumArg<'a> {
  Node(Option<Node<'a>>),
  Num(f64),
}

fn number_arg(n: NumArg<'_>) -> LResult<f64> {
  match n {
    NumArg::Num(x) => Ok(x),
    NumArg::Node(n) => number(n),
  }
}

fn scaled(n: NumArg<'_>, size: f64) -> LResult<f64> {
  if let NumArg::Node(Some(Node::Dimension(d))) = n
    && is_percent(d.unit)
  {
    return Ok(d.value * size / 100.0);
  }
  number_arg(n)
}

fn rgba_impl<'a>(
  c: &mut EvalCtx<'a, '_>,
  r: NumArg<'a>,
  g: NumArg<'a>,
  b: NumArg<'a>,
  a: NumArg<'a>,
) -> Option<&'a Color<'a>> {
  let res: LResult<&'a Color<'a>> = (|| {
    if let NumArg::Node(Some(Node::Color(col))) = r {
      let alpha = match g {
        NumArg::Node(None) => col.alpha,
        other => number_arg(other)?,
      };
      return Ok(c.a.color(col.rgb, alpha, Some("rgba")));
    }
    let rgb = [scaled(r, 255.0)?, scaled(g, 255.0)?, scaled(b, 255.0)?];
    let alpha = number_arg(a)?;
    Ok(c.a.color(rgb, alpha, Some("rgba")))
  })();
  res.ok()
}

fn split_space_args<'a>(a: &[Node<'a>]) -> (NumArg<'a>, NumArg<'a>, NumArg<'a>, NumArg<'a>) {
  let first = a.first().copied();
  if let Some(Node::Expression(e)) = first {
    let v = e.value;
    let r = v.first().copied();
    let g = v.get(1).copied();
    let mut b = v.get(2).copied();
    let mut alpha = NumArg::Num(1.0);
    if let Some(Node::Operation(op)) = b {
      b = Some(op.operands[0]);
      alpha = NumArg::Node(Some(op.operands[1]));
    }
    return (NumArg::Node(r), NumArg::Node(g), NumArg::Node(b), alpha);
  }
  (
    NumArg::Node(first),
    NumArg::Node(a.get(1).copied()),
    NumArg::Node(a.get(2).copied()),
    NumArg::Num(1.0),
  )
}

fn rgb<'a>(c: &mut EvalCtx<'a, '_>, _p: Pos, a: &[Node<'a>]) -> R<'a> {
  let (r, g, b, alpha) = split_space_args(a);
  match rgba_impl(c, r, g, b, alpha) {
    Some(col) => {
      col.value.set(Some("rgb"));
      Ok(Some(Node::Color(col)))
    }
    None => Ok(None),
  }
}

fn rgba<'a>(c: &mut EvalCtx<'a, '_>, _p: Pos, a: &[Node<'a>]) -> R<'a> {
  let arg = |i: usize| NumArg::Node(a.get(i).copied());
  Ok(rgba_impl(c, arg(0), arg(1), arg(2), arg(3)).map(Node::Color))
}

fn hsla_impl<'a>(
  c: &mut EvalCtx<'a, '_>,
  h: NumArg<'a>,
  s: NumArg<'a>,
  l: NumArg<'a>,
  a: NumArg<'a>,
) -> Option<&'a Color<'a>> {
  let res: LResult<&'a Color<'a>> = (|| {
    if let NumArg::Node(Some(Node::Color(col))) = h {
      let alpha = match s {
        NumArg::Node(None) => col.alpha,
        other => number_arg(other)?,
      };
      return Ok(c.a.color(col.rgb, alpha, Some("hsla")));
    }
    let h = (number_arg(h)? % 360.0) / 360.0;
    let s = clamp01(number_arg(s)?);
    let l = clamp01(number_arg(l)?);
    let a = clamp01(number_arg(a)?);
    let m2 = if l <= 0.5 {
      l * (s + 1.0)
    } else {
      l + s - l * s
    };
    let m1 = l * 2.0 - m2;
    let hue = |h: f64| -> f64 {
      let h = if h < 0.0 {
        h + 1.0
      } else if h > 1.0 {
        h - 1.0
      } else {
        h
      };
      if h * 6.0 < 1.0 {
        m1 + (m2 - m1) * h * 6.0
      } else if h * 2.0 < 1.0 {
        m2
      } else if h * 3.0 < 2.0 {
        m1 + (m2 - m1) * (2.0 / 3.0 - h) * 6.0
      } else {
        m1
      }
    };
    let rgb = [
      hue(h + 1.0 / 3.0) * 255.0,
      hue(h) * 255.0,
      hue(h - 1.0 / 3.0) * 255.0,
    ];
    Ok(c.a.color(rgb, a, Some("hsla")))
  })();
  res.ok()
}

fn hsl<'a>(c: &mut EvalCtx<'a, '_>, _p: Pos, a: &[Node<'a>]) -> R<'a> {
  let (h, s, l, alpha) = split_space_args(a);
  match hsla_impl(c, h, s, l, alpha) {
    Some(col) => {
      col.value.set(Some("hsl"));
      Ok(Some(Node::Color(col)))
    }
    None => Ok(None),
  }
}

fn hsla_fn<'a>(c: &mut EvalCtx<'a, '_>, _p: Pos, a: &[Node<'a>]) -> R<'a> {
  let arg = |i: usize| NumArg::Node(a.get(i).copied());
  Ok(hsla_impl(c, arg(0), arg(1), arg(2), arg(3)).map(Node::Color))
}

/// The `hsla(origColor, hsl)` helper.
fn hsla_from<'a>(c: &mut EvalCtx<'a, '_>, orig: Node<'a>, h: Hsl) -> Option<Node<'a>> {
  let col = hsla_impl(
    c,
    NumArg::Num(h.h),
    NumArg::Num(h.s),
    NumArg::Num(h.l),
    NumArg::Num(h.a),
  )?;
  let ov = match orig {
    Node::Color(o) => o.value.get(),
    _ => None,
  };
  match ov {
    Some(v) if v.starts_with("rgb") || v.starts_with("hsl") => col.value.set(Some(v)),
    _ => col.value.set(Some("rgb")),
  }
  Some(Node::Color(col))
}

fn hsva_impl<'a>(
  c: &mut EvalCtx<'a, '_>,
  h: NumArg<'a>,
  s: NumArg<'a>,
  v: NumArg<'a>,
  a: NumArg<'a>,
) -> R<'a> {
  let h = ((number_arg(h)? % 360.0) / 360.0) * 360.0;
  let s = number_arg(s)?;
  let v = number_arg(v)?;
  let a = number_arg(a)?;
  let i = ((h / 60.0) % 6.0).floor();
  let f = (h / 60.0) - i;
  let vs = [
    v,
    v * (1.0 - s),
    v * (1.0 - f * s),
    v * (1.0 - (1.0 - f) * s),
  ];
  let perm = [
    [0, 3, 1],
    [2, 0, 1],
    [1, 0, 3],
    [1, 2, 0],
    [3, 1, 0],
    [0, 1, 2],
  ];
  if i.is_nan() || !(0.0..6.0).contains(&i) {
    return Err(undefined_value("0"));
  }
  let p = perm[i as usize];
  Ok(
    rgba_impl(
      c,
      NumArg::Num(vs[p[0]] * 255.0),
      NumArg::Num(vs[p[1]] * 255.0),
      NumArg::Num(vs[p[2]] * 255.0),
      NumArg::Num(a),
    )
    .map(Node::Color),
  )
}

fn hsv<'a>(c: &mut EvalCtx<'a, '_>, _p: Pos, a: &[Node<'a>]) -> R<'a> {
  let arg = |i: usize| NumArg::Node(a.get(i).copied());
  hsva_impl(c, arg(0), arg(1), arg(2), NumArg::Num(1.0))
}

fn hsva_fn<'a>(c: &mut EvalCtx<'a, '_>, _p: Pos, a: &[Node<'a>]) -> R<'a> {
  let arg = |i: usize| NumArg::Node(a.get(i).copied());
  hsva_impl(c, arg(0), arg(1), arg(2), arg(3))
}

#[derive(Copy, Clone)]
enum Adjust {
  S,
  L,
  A,
}

fn adjust<'a>(
  c: &mut EvalCtx<'a, '_>,
  a: &[Node<'a>],
  which: Adjust,
  sign: f64,
  check_rgb: bool,
) -> R<'a> {
  if check_rgb && !matches!(a.first(), Some(Node::Color(_))) {
    if a.is_empty() {
      return Err(undefined_value("rgb"));
    }
    return Ok(None);
  }
  let x = col(a, 0)?;
  let mut h = to_hsl(x);
  let amount = match a.get(1) {
    Some(n) => super::builtins::num_value(Some(*n))?,
    None => return Err(undefined_value("value")),
  };
  let relative =
    matches!(a.get(2), Some(m) if crate::eval::js_string_value(*m) == Some("relative"));
  let field = match which {
    Adjust::S => &mut h.s,
    Adjust::L => &mut h.l,
    Adjust::A => &mut h.a,
  };
  if relative {
    *field += sign * *field * amount / 100.0;
  } else {
    *field += sign * amount / 100.0;
  }
  *field = clamp01(*field);
  Ok(hsla_from(c, Node::Color(x), h))
}

fn fade<'a>(c: &mut EvalCtx<'a, '_>, _p: Pos, a: &[Node<'a>]) -> R<'a> {
  let x = col(a, 0)?;
  let mut h = to_hsl(x);
  let amount = super::builtins::num_value(a.get(1).copied())?;
  h.a = clamp01(amount / 100.0);
  Ok(hsla_from(c, Node::Color(x), h))
}

fn spin<'a>(c: &mut EvalCtx<'a, '_>, _p: Pos, a: &[Node<'a>]) -> R<'a> {
  let x = col(a, 0)?;
  let mut h = to_hsl(x);
  let amount = super::builtins::num_value(a.get(1).copied())?;
  let hue = (h.h + amount) % 360.0;
  h.h = if hue < 0.0 { 360.0 + hue } else { hue };
  Ok(hsla_from(c, Node::Color(x), h))
}

fn mix<'a>(c: &mut EvalCtx<'a, '_>, _p: Pos, a: &[Node<'a>]) -> R<'a> {
  let c1 = a.first().copied().ok_or_else(|| undefined_value("toHSL"))?;
  mix_impl(c, c1, a.get(1).copied(), a.get(2).copied())
}

fn mix_impl<'a>(
  c: &mut EvalCtx<'a, '_>,
  c1: Node<'a>,
  c2: Option<Node<'a>>,
  weight: Option<Node<'a>>,
) -> R<'a> {
  let color1 = match c1 {
    Node::Color(x) => x,
    _ => return Err(LessError::native("Argument cannot be evaluated to a color")),
  };
  let color2 = match c2 {
    Some(Node::Color(x)) => x,
    Some(_) => return Err(LessError::native("Argument cannot be evaluated to a color")),
    None => return Err(undefined_value("toHSL")),
  };
  let wv = match weight {
    Some(w) => super::builtins::num_value(Some(w))?,
    None => 50.0,
  };
  let p = wv / 100.0;
  let w = p * 2.0 - 1.0;
  let aa = to_hsl(color1).a - to_hsl(color2).a;
  let w1 = ((if w * aa == -1.0 {
    w
  } else {
    (w + aa) / (1.0 + w * aa)
  }) + 1.0)
    / 2.0;
  let w2 = 1.0 - w1;
  let rgb = [
    color1.rgb[0] * w1 + color2.rgb[0] * w2,
    color1.rgb[1] * w1 + color2.rgb[1] * w2,
    color1.rgb[2] * w1 + color2.rgb[2] * w2,
  ];
  let alpha = color1.alpha * p + color2.alpha * (1.0 - p);
  Ok(Some(Node::Color(c.a.color(rgb, alpha, None))))
}

fn contrast<'a>(c: &mut EvalCtx<'a, '_>, _p: Pos, a: &[Node<'a>]) -> R<'a> {
  let color = match a.first() {
    Some(Node::Color(x)) => *x,
    Some(_) => return Ok(None),
    None => return Err(undefined_value("rgb")),
  };
  let mut dark: Node<'a> = match a.get(1) {
    Some(d) => *d,
    None => Node::Color(c.a.color([0.0, 0.0, 0.0], 1.0, Some("rgba"))),
  };
  let mut light: Node<'a> = match a.get(2) {
    Some(l) => *l,
    None => Node::Color(c.a.color([255.0, 255.0, 255.0], 1.0, Some("rgba"))),
  };
  let luma_of = |n: Node<'a>| -> LResult<f64> {
    match n {
      Node::Color(x) => Ok(color::luma(x)),
      _ => Err(LessError::native("dark.luma is not a function")),
    }
  };
  if luma_of(dark)? > luma_of(light)? {
    std::mem::swap(&mut dark, &mut light);
  }
  let threshold = match a.get(3) {
    Some(t) => number(Some(*t))?,
    None => 0.43,
  };
  Ok(Some(if color::luma(color) < threshold {
    light
  } else {
    dark
  }))
}

fn color_fn<'a>(c: &mut EvalCtx<'a, '_>, _p: Pos, a: &[Node<'a>]) -> R<'a> {
  let arg = a.first().copied();
  if let Some(Node::Quoted(q)) = arg {
    let v = q.value;
    let b = v.as_bytes();
    if b.first() == Some(&b'#') {
      let hex = &v[1..];
      let n = hex.len();
      if (n == 3 || n == 4 || n == 6 || n == 8) && hex.bytes().all(|x| x.is_ascii_hexdigit()) {
        let original = c.str(&format!("#{}", hex));
        return Ok(Some(Node::Color(color_from_hex(c.a, hex, Some(original)))));
      }
    }
  }
  match arg {
    Some(Node::Color(col)) => {
      col.value.set(None);
      return Ok(Some(Node::Color(col)));
    }
    Some(other) => {
      if let Some(s) = crate::eval::js_string_value(other)
        && let Some(Node::Color(col)) = color_from_keyword(c.a, s)
      {
        col.value.set(None);
        return Ok(Some(Node::Color(col)));
      }
    }
    None => return Err(undefined_value("value")),
  }
  Err(LessError::new(
    "Argument",
    "argument must be a color keyword or 3|4|6|8 digit hex e.g. #FFF",
  ))
}

fn blend<'a>(c: &mut EvalCtx<'a, '_>, a: &[Node<'a>], mode: impl Fn(f64, f64) -> f64) -> R<'a> {
  let c1 = match a.first() {
    Some(Node::Color(x)) => *x,
    _ => return Err(undefined_value("alpha")),
  };
  let c2 = match a.get(1) {
    Some(Node::Color(x)) => *x,
    _ => return Err(undefined_value("alpha")),
  };
  let ab = c1.alpha;
  let as_ = c2.alpha;
  let ar = as_ + ab * (1.0 - as_);
  let mut r = [0.0; 3];
  for i in 0..3 {
    let cb = c1.rgb[i] / 255.0;
    let cs = c2.rgb[i] / 255.0;
    let mut cr = mode(cb, cs);
    if ar != 0.0 && !ar.is_nan() {
      cr = (as_ * cs + ab * (cb - as_ * (cb + cs - cr))) / ar;
    }
    r[i] = cr * 255.0;
  }
  Ok(Some(Node::Color(c.a.color(r, ar, None))))
}

/// `unit.toString().toUpperCase() === '%'`
fn is_percent(u: &Unit<'_>) -> bool {
  u.denominator.is_empty() && matches!(u.numerator, ["%"])
}
