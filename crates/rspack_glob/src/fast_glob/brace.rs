#[derive(Debug, Default, Clone)]
pub(crate) struct Pattern {
  pub value: Vec<char>,
  pub branch: Vec<(u8, u8)>,
  pub shadow: Vec<(usize, usize)>,
}

impl Pattern {
  pub fn parse(glob: &[char]) -> Option<Vec<(u8, u8)>> {
    let mut depth = 0;
    let mut current = 0;
    let mut in_brackets = false;

    let mut stack = [0; 10];
    let mut branch = Vec::<(u8, u8)>::with_capacity(16);

    while current < glob.len() {
      match glob[current] {
        '\\' => current += 1,
        ']' if in_brackets => in_brackets = false,
        '[' if !in_brackets => in_brackets = true,
        ',' if !in_brackets && depth > 0 => {
          branch[stack[depth - 1]].1 += 1;
        }
        '}' if !in_brackets && depth > 0 => {
          depth -= 1;
        }
        '{' if !in_brackets => {
          branch.push((0, 1));

          stack[depth] = branch.len() - 1;
          depth += 1;
        }
        _ => {}
      }
      current += 1;
    }

    if depth == 0 && !in_brackets {
      Some(branch)
    } else {
      None
    }
  }

  pub fn new(glob: &[char]) -> Option<Self> {
    if let Some(branch) = Self::parse(glob) {
      if branch.is_empty() {
        let value = Vec::new();
        let shadow = Vec::new();

        return Some(Pattern {
          value,
          branch,
          shadow,
        });
      }

      let value = Vec::with_capacity(glob.len());
      let shadow = Vec::with_capacity(branch.len());

      let mut pattern = Pattern {
        value,
        branch,
        shadow,
      };

      pattern.track(glob);
      return Some(pattern);
    }
    None
  }

  pub fn track(&mut self, glob: &[char]) {
    let mut index = 0;

    let mut depth = 0;
    let mut current = 0;
    let mut is_valid = true;
    let mut in_brackets = false;

    let mut len = 0;
    let mut stack: [(u8, usize); 10] = [(0, 0); 10];

    self.value.clear();
    self.shadow.clear();

    while current < glob.len() {
      match glob[current] {
        ',' if !in_brackets && depth > 0 => {
          if len == depth {
            let (i, idx) = &mut stack[len - 1];

            *i += 1;
            is_valid = self.branch[*idx].0 == *i;
          }
        }
        '}' if !in_brackets && depth > 0 => {
          if len == depth {
            len -= 1;
            is_valid = true;
          }
          depth -= 1;
        }
        '{' if !in_brackets => {
          if is_valid {
            stack[len] = (0, index);

            len += 1;
            is_valid = self.branch[index].0 == 0;

            self.shadow.push((index, self.value.len()));
          }

          depth += 1;
          index += 1;
        }
        c => {
          if is_valid {
            self.value.push(c);
          }

          if c == '\\' {
            current += 1;
            if is_valid && current < glob.len() {
              self.value.push(glob[current]);
            }
          } else if c == ']' && in_brackets {
            in_brackets = false;
          } else if c == '[' && !in_brackets {
            in_brackets = true;
          }
        }
      }

      current += 1;
    }
  }

  pub fn trigger(&mut self, glob: &[char], target: usize) -> bool {
    for &(idx, position) in self.shadow.iter().rev() {
      if target >= position {
        self.branch[idx].0 += 1;
        if self.branch[idx].1 != self.branch[idx].0 {
          self.track(glob);
          return true;
        }
        self.branch[idx].0 = 0;
      }
    }
    false
  }

  pub fn restore(&mut self) {
    for b in &mut self.branch {
      if b.0 != 0 {
        b.0 = 0;
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn brace_expansion() {
    let glob = "some/{a,b{c,d}f,e}/ccc.{png,jpg}"
      .chars()
      .collect::<Vec<_>>();
    let mut pattern = Pattern::new(&glob).unwrap();

    loop {
      if !pattern.trigger(&glob, pattern.value.len()) {
        break;
      }
    }
  }
}
