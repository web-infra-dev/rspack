use once_cell::sync::Lazy;
use regex::Regex;

static EXACT_REG: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[^*!]+$").expect("TODO:"));

static CONTAIN_REG: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\*.+\*$").expect("TODO:"));
static ENDS_WITH_REG: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\*[^*]+$").expect("TODO:"));
static STARTS_WITH_REG: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[^*!]+\*$").expect("TODO:"));
static NOT_EXACT: Lazy<Regex> = Lazy::new(|| Regex::new(r"^![^*].*$").expect("TODO:"));
static NOT_CONTAIN_REG: Lazy<Regex> = Lazy::new(|| Regex::new(r"^!\*.+\*$").expect("TODO:"));
static NOT_ENDS_WITH_REG: Lazy<Regex> = Lazy::new(|| Regex::new(r"^!\*[^*]+$").expect("TODO:"));
static NOT_STARTS_WITH_REG: Lazy<Regex> = Lazy::new(|| Regex::new(r"^![^*]+\*").expect("TODO:"));

pub fn exact(list: &[String]) -> Vec<String> {
  list
    .iter()
    .filter(|prop| EXACT_REG.is_match(prop))
    .map(|a| a.into())
    .collect::<Vec<_>>()
}

pub fn contain(list: &[String]) -> Vec<String> {
  list
    .iter()
    .filter(|prop| CONTAIN_REG.is_match(prop))
    .map(|prop| (&prop[1..prop.len() - 1]).into())
    .collect::<Vec<_>>()
}

pub fn ends_with(list: &[String]) -> Vec<String> {
  list
    .iter()
    .filter(|prop| ENDS_WITH_REG.is_match(prop))
    .map(|prop| (&prop[1..]).into())
    .collect::<Vec<_>>()
}

pub fn starts_with(list: &[String]) -> Vec<String> {
  list
    .iter()
    .filter(|prop| STARTS_WITH_REG.is_match(prop))
    .map(|prop| (&prop[0..prop.len() - 1]).into())
    .collect::<Vec<_>>()
}

pub fn not_exact(list: &[String]) -> Vec<String> {
  list
    .iter()
    .filter(|prop| NOT_EXACT.is_match(prop))
    .map(|prop| (&prop[1..]).into())
    .collect::<Vec<_>>()
}

pub fn not_contain(list: &[String]) -> Vec<String> {
  list
    .iter()
    .filter(|prop| NOT_CONTAIN_REG.is_match(prop))
    .map(|prop| (&prop[2..prop.len() - 1]).into())
    .collect::<Vec<_>>()
}

pub fn not_ends_with(list: &[String]) -> Vec<String> {
  list
    .iter()
    .filter(|prop| NOT_ENDS_WITH_REG.is_match(prop))
    .map(|prop| (&prop[2..]).into())
    .collect::<Vec<_>>()
}

pub fn not_starts_with(list: &[String]) -> Vec<String> {
  list
    .iter()
    .filter(|prop| NOT_STARTS_WITH_REG.is_match(prop))
    .map(|prop| (&prop[1..prop.len() - 1]).into())
    .collect::<Vec<_>>()
}
