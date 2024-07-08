use rspack_macros::{plugin, plugin_hook};

mod mock_hook {
  pub trait AsyncSeries<T, R> {
    fn run(&self, a: T) -> R;
    fn stage(&self) -> i32 {
      0
    }
  }
}

mod mock_core {
  #[derive(Debug)]
  pub struct Compilation;

  pub const BASIC_STAGE: i32 = 20;
}

mod named_struct {
  use super::*;

  #[plugin]
  #[derive(Debug, Clone)]
  struct Plugin {
    options: String,
    state: usize,
  }

  #[plugin_hook(mock_hook::AsyncSeries<mock_core::Compilation, String> for Plugin)]
  fn process_assets(&self, _a: mock_core::Compilation) -> String {
    format!("process_assets {} {}", self.options, self.state)
  }

  #[plugin_hook(mock_hook::AsyncSeries<mock_core::Compilation, String> for Plugin, stage = 100)]
  fn make(&self, _a: mock_core::Compilation) -> String {
    format!("make {} {}", self.options, self.state)
  }

  #[plugin_hook(mock_hook::AsyncSeries<mock_core::Compilation, String> for Plugin, stage = mock_core::BASIC_STAGE)]
  fn compilation(&self, _a: mock_core::Compilation) -> String {
    format!("compilation {} {}", self.options, self.state)
  }

  #[test]
  fn test() {
    let plugin = Plugin::new_inner("aa".to_string(), 0);
    let hook1 = &compilation::new(&plugin);
    let hook2 = &make::new(&plugin);
    let hook3 = &process_assets::new(&plugin);
    let r = mock_hook::AsyncSeries::run(hook1, mock_core::Compilation);
    assert_eq!(r, "compilation aa 0");
    let s = mock_hook::AsyncSeries::stage(hook1);
    assert_eq!(s, 20);
    let r = mock_hook::AsyncSeries::run(hook2, mock_core::Compilation);
    assert_eq!(r, "make aa 0");
    let s = mock_hook::AsyncSeries::stage(hook2);
    assert_eq!(s, 100);
    let r = mock_hook::AsyncSeries::run(hook3, mock_core::Compilation);
    assert_eq!(r, "process_assets aa 0");
    let s = mock_hook::AsyncSeries::stage(hook3);
    assert_eq!(s, 0);
  }
}

mod unit_struct {
  use super::*;

  #[plugin]
  #[derive(Debug, Default, Clone)]
  struct Plugin;

  #[plugin_hook(mock_hook::AsyncSeries<mock_core::Compilation, String> for Plugin)]
  fn process_assets(&self, _a: mock_core::Compilation) -> String {
    "process_assets".to_string()
  }

  #[plugin_hook(mock_hook::AsyncSeries<mock_core::Compilation, String> for Plugin, stage = 100)]
  fn make(&self, _a: mock_core::Compilation) -> String {
    "make".to_string()
  }

  #[plugin_hook(mock_hook::AsyncSeries<mock_core::Compilation, String> for Plugin, stage = mock_core::BASIC_STAGE)]
  fn compilation(&self, _a: mock_core::Compilation) -> String {
    "compilation".to_string()
  }

  #[test]
  fn test() {
    let plugin = Plugin::default();
    let hook1 = &compilation::new(&plugin);
    let hook2 = &make::new(&plugin);
    let hook3 = &process_assets::new(&plugin);
    let r = mock_hook::AsyncSeries::run(hook1, mock_core::Compilation);
    assert_eq!(r, "compilation");
    let s = mock_hook::AsyncSeries::stage(hook1);
    assert_eq!(s, 20);
    let r = mock_hook::AsyncSeries::run(hook2, mock_core::Compilation);
    assert_eq!(r, "make");
    let s = mock_hook::AsyncSeries::stage(hook2);
    assert_eq!(s, 100);
    let r = mock_hook::AsyncSeries::run(hook3, mock_core::Compilation);
    assert_eq!(r, "process_assets");
    let s = mock_hook::AsyncSeries::stage(hook3);
    assert_eq!(s, 0);
  }
}
