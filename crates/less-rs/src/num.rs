//! ECMAScript number formatting.
//!
//! less.js prints numbers with `String(x)` and rounds them with
//! `Number((x + 2e-16).toFixed(8))`. Both have to be reproduced exactly for the
//! output to be byte-identical, so this module implements the algorithms from
//! the ECMAScript specification (`Number::toString` and
//! `Number.prototype.toFixed`) on top of Rust's shortest round-trip digit
//! generation.

use std::fmt::Write;

/// `String(x)` / `Number.prototype.toString()` for a JS number.
pub fn to_js_string(x: f64) -> String {
  let mut s = String::new();
  write_js(&mut s, x);
  s
}

/// Appends `String(x)` to `out`.
pub fn write_js(out: &mut String, x: f64) {
  if x.is_nan() {
    out.push_str("NaN");
    return;
  }
  if x == 0.0 {
    out.push('0');
    return;
  }
  if x.is_infinite() {
    out.push_str(if x > 0.0 { "Infinity" } else { "-Infinity" });
    return;
  }
  // Fast path for integers that are exactly representable.
  if x.fract() == 0.0 && x.abs() < 1e21 && x.abs() < 9.007_199_254_740_992e15 {
    let _ = write!(out, "{}", x as i64);
    return;
  }
  let neg = x < 0.0;
  let x = x.abs();
  // Shortest round-trip digits (ryu), then the ECMAScript layout.
  let mut buf = ryu::Buffer::new();
  let s = buf.format_finite(x).as_bytes();
  let mut digits = [0u8; 32];
  let mut k = 0usize;
  let mut point: Option<usize> = None;
  let mut exp: i32 = 0;
  let mut i = 0;
  while i < s.len() {
    let c = s[i];
    if c.is_ascii_digit() {
      digits[k] = c;
      k += 1;
    } else if c == b'.' {
      point = Some(k);
    } else if c == b'e' || c == b'E' {
      let mut j = i + 1;
      let mut negexp = false;
      if j < s.len() && (s[j] == b'-' || s[j] == b'+') {
        negexp = s[j] == b'-';
        j += 1;
      }
      while j < s.len() {
        exp = exp * 10 + (s[j] - b'0') as i32;
        j += 1;
      }
      if negexp {
        exp = -exp;
      }
      break;
    }
    i += 1;
  }
  // value = 0.d1..dk * 10^n
  let int_len = point.unwrap_or(k) as i32;
  let mut n = int_len + exp;
  // strip leading zeros
  let mut start = 0;
  while start + 1 < k && digits[start] == b'0' {
    start += 1;
    n -= 1;
  }
  // strip trailing zeros
  let mut end = k;
  while end > start + 1 && digits[end - 1] == b'0' {
    end -= 1;
  }
  let d = &digits[start..end];
  let k = d.len() as i32;
  let ds = std::str::from_utf8(d).unwrap_or("0");
  if neg {
    out.push('-');
  }
  if k <= n && n <= 21 {
    out.push_str(ds);
    for _ in 0..(n - k) {
      out.push('0');
    }
  } else if 0 < n && n <= 21 {
    out.push_str(&ds[..n as usize]);
    out.push('.');
    out.push_str(&ds[n as usize..]);
  } else if -6 < n && n <= 0 {
    out.push_str("0.");
    for _ in 0..(-n) {
      out.push('0');
    }
    out.push_str(ds);
  } else {
    let e = n - 1;
    out.push_str(&ds[..1]);
    if k > 1 {
      out.push('.');
      out.push_str(&ds[1..]);
    }
    out.push('e');
    out.push(if e < 0 { '-' } else { '+' });
    let _ = write!(out, "{}", e.abs());
  }
}

/// `Number.prototype.toFixed(digits)`.
pub fn to_fixed(x: f64, digits: u32) -> String {
  if x.is_nan() {
    return "NaN".to_string();
  }
  if x.abs() >= 1e21 || x.is_infinite() {
    return to_js_string(x);
  }
  let neg = x < 0.0;
  let ax = x.abs();
  let mut body = if digits <= 20 {
    fixed_exact(ax, digits)
  } else {
    // Very rare; fall back to Rust's exact formatting (ties to even).
    format!("{:.*}", digits as usize, ax)
  };
  if neg {
    body.insert(0, '-');
  }
  body
}

/// Exact `toFixed` for a non-negative finite value below 1e21 using integer
/// arithmetic on the binary representation; ties round to the larger value as
/// the specification requires.
fn fixed_exact(x: f64, digits: u32) -> String {
  let bits = x.to_bits();
  let exp_bits = ((bits >> 52) & 0x7ff) as i32;
  let frac = bits & ((1u64 << 52) - 1);
  let (m, e) = if exp_bits == 0 {
    (frac, -1074)
  } else {
    (frac | (1u64 << 52), exp_bits - 1075)
  };
  let scaled: u128 = if m == 0 {
    0
  } else if e >= 0 {
    // Integral value: no rounding needed.
    let int = (m as u128) << e;
    let mut s = int.to_string();
    if digits > 0 {
      s.push('.');
      for _ in 0..digits {
        s.push('0');
      }
    }
    return s;
  } else {
    let shift = (-e) as u32;
    let num = (m as u128) * 10u128.pow(digits);
    if shift >= 128 {
      0
    } else {
      let q = num >> shift;
      let r = num - (q << shift);
      let half = 1u128 << (shift - 1);
      if r >= half { q + 1 } else { q }
    }
  };
  let mut s = scaled.to_string();
  if digits == 0 {
    return s;
  }
  let d = digits as usize;
  if s.len() <= d {
    let pad = d + 1 - s.len();
    s.insert_str(0, &"0".repeat(pad));
  }
  let point = s.len() - d;
  s.insert(point, '.');
  s
}

/// `Number((value + 2e-16).toFixed(precision))` as used by `Node.fround`.
pub fn fround(value: f64, precision: Option<u32>) -> f64 {
  match precision {
    Some(p) if p > 0 => {
      let x = value + 2e-16;
      if x.is_finite() && x.abs() < 1e7 && p <= 8 {
        // The rounded decimal `n / 10^p` with n < 2^53: dividing two
        // exact doubles is correctly rounded, i.e. equals parsing the
        // decimal string.
        if let Some(n) = fixed_scaled(x.abs(), p)
          && n < (1u128 << 53)
        {
          let r = n as f64 / 10f64.powi(p as i32);
          return if x < 0.0 { -r } else { r };
        }
      }
      to_fixed(x, p).parse::<f64>().unwrap_or(value)
    }
    _ => value,
  }
}

/// `Number(x.toFixed(digits))` without building the string: the exact
/// rounded decimal `n / 10^digits` with `n < 2^53` and `10^digits` exact is
/// what parsing the string yields (both are correctly rounded).
pub fn to_fixed_number(x: f64, digits: u32) -> f64 {
  if x.is_finite()
    && x.abs() < 1e21
    && digits <= 22
    && let Some(n) = fixed_scaled(x.abs(), digits)
    && n < (1u128 << 53)
  {
    let r = n as f64 / 10f64.powi(digits as i32);
    return if x < 0.0 { -r } else { r };
  }
  parse_float(&to_fixed(x, digits))
}

/// `round_half_up(x * 10^digits)` computed exactly (x ≥ 0, finite).
fn fixed_scaled(x: f64, digits: u32) -> Option<u128> {
  let bits = x.to_bits();
  let exp_bits = ((bits >> 52) & 0x7ff) as i32;
  let frac = bits & ((1u64 << 52) - 1);
  let (m, e) = if exp_bits == 0 {
    (frac, -1074)
  } else {
    (frac | (1u64 << 52), exp_bits - 1075)
  };
  if m == 0 {
    return Some(0);
  }
  if e >= 0 {
    return ((m as u128) << e).checked_mul(10u128.pow(digits));
  }
  let shift = (-e) as u32;
  let num = (m as u128) * 10u128.pow(digits);
  if shift >= 128 {
    return Some(0);
  }
  let q = num >> shift;
  let r = num - (q << shift);
  let half = 1u128 << (shift - 1);
  Some(if r >= half { q + 1 } else { q })
}

/// JS `parseFloat` for the prefix of a string.
pub fn parse_float(s: &str) -> f64 {
  let t = s.trim_start();
  let b = t.as_bytes();
  let mut i = 0;
  if i < b.len() && (b[i] == b'+' || b[i] == b'-') {
    i += 1;
  }
  if t[i..].starts_with("Infinity") {
    return if b.first() == Some(&b'-') {
      f64::NEG_INFINITY
    } else {
      f64::INFINITY
    };
  }
  let start_digits = i;
  while i < b.len() && b[i].is_ascii_digit() {
    i += 1;
  }
  if i < b.len() && b[i] == b'.' {
    i += 1;
    while i < b.len() && b[i].is_ascii_digit() {
      i += 1;
    }
  }
  if i == start_digits || (i == start_digits + 1 && b[start_digits] == b'.') {
    return f64::NAN;
  }
  let mantissa_end = i;
  if i < b.len() && (b[i] == b'e' || b[i] == b'E') {
    let mut j = i + 1;
    if j < b.len() && (b[j] == b'+' || b[j] == b'-') {
      j += 1;
    }
    let ds = j;
    while j < b.len() && b[j].is_ascii_digit() {
      j += 1;
    }
    if j > ds {
      i = j;
    }
  }
  let end = if i > mantissa_end { i } else { mantissa_end };
  t[..end].parse::<f64>().unwrap_or(f64::NAN)
}

/// JS `Math.round`: rounds half up (towards +Infinity).
pub fn js_round(x: f64) -> f64 {
  if x.is_nan() || x.is_infinite() {
    return x;
  }
  let f = x.floor();
  if x - f >= 0.5 { f + 1.0 } else { f }
}
