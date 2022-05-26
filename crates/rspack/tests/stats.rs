mod utils;

use utils::compile_to_get_stats;

#[test]
fn stats_example() {
  let stats = compile_to_get_stats("single-entry", Default::default(), vec![]);
  assert!(stats.map.contains_key("main.js"));
}
