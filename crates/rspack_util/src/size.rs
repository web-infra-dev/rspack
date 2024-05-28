pub fn format_size(size: f64) -> String {
  if !size.is_finite() {
    return String::from("unknown size");
  }

  if size <= 0.0 {
    return String::from("0 bytes");
  }

  let abbreviations = ["bytes", "KiB", "MiB", "GiB"];
  let mut index = size.log(1024.0).floor() as usize;

  if index >= abbreviations.len() {
    index = abbreviations.len() - 1;
  }

  format!(
    "{:.3} {}",
    size / 1024.0_f64.powf(index as f64),
    abbreviations[index]
  )
}
