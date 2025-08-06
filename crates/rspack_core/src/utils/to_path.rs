fn normalize_path_chars(s: &str) -> String {
  let mut result = String::with_capacity(s.len());
  let mut last_was_replaced = false;
  
  for ch in s.chars() {
    let is_allowed = match ch {
      'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '!' | '§' | '$' | '(' | ')' | '-' | '=' | '^' | '°' => true,
      _ => false,
    };
    
    if is_allowed {
      result.push(ch);
      last_was_replaced = false;
    } else if !last_was_replaced {
      result.push('-');
      last_was_replaced = true;
    }
    // If not allowed and last was already replaced, skip this char
  }
  
  result
}

fn trim_hyphens(s: &str) -> &str {
  let start = s.find(|c: char| c != '-').unwrap_or(s.len());
  let end = s.rfind(|c: char| c != '-').map(|i| i + 1).unwrap_or(0);
  
  if end > start {
    &s[start..end]
  } else {
    ""
  }
}

pub fn to_path(str: &str) -> String {
  let temp = normalize_path_chars(str);
  let res = trim_hyphens(&temp);
  res.to_string()
}
