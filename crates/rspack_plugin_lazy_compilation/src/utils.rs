const VALUE_DEP_PREFIX: &str = "rspack/LazyCompilation ";

pub fn calc_value_dependency_key(key: &str) -> String {
  format!("{VALUE_DEP_PREFIX}{key}")
}
