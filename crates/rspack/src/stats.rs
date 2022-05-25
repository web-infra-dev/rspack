use std::{collections::HashMap, time::Instant};

#[derive(Debug)]
pub struct Stats {
  pub map: HashMap<String, String>,
  pub start_time: Instant,
  pub end_time: Instant,
}
