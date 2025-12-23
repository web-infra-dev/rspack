use rspack_browserslist::load_browserslist;
use rspack_core::{CompilerPlatform, Context};

/// Targets type.
pub type Targets = Vec<String>;

#[derive(Debug, Default)]
pub struct TargetProperties {
  pub web: Option<bool>,
  pub browser: Option<bool>,
  pub webworker: Option<bool>,
  pub node: Option<bool>,
  pub electron: Option<bool>,
  pub nwjs: Option<bool>,
  pub electron_main: Option<bool>,
  pub electron_preload: Option<bool>,
  pub electron_renderer: Option<bool>,
  pub require: Option<bool>,
  pub node_builtins: Option<bool>,
  pub node_prefix_for_core_modules: Option<bool>,
  pub document: Option<bool>,
  pub import_scripts: Option<bool>,
  pub import_scripts_in_worker: Option<bool>,
  pub fetch_wasm: Option<bool>,
  pub global: Option<bool>,
  pub global_this: Option<bool>,
  pub big_int_literal: Option<bool>,
  pub r#const: Option<bool>,
  pub arrow_function: Option<bool>,
  pub for_of: Option<bool>,
  pub destructuring: Option<bool>,
  pub dynamic_import: Option<bool>,
  pub dynamic_import_in_worker: Option<bool>,
  pub module: Option<bool>,
  pub optional_chaining: Option<bool>,
  pub template_literal: Option<bool>,
  pub async_function: Option<bool>,
}

#[allow(unused)]
impl TargetProperties {
  pub fn web(&self) -> bool {
    self.web.unwrap_or(false)
  }
  pub fn browser(&self) -> bool {
    self.browser.unwrap_or(false)
  }
  pub fn webworker(&self) -> bool {
    self.webworker.unwrap_or(false)
  }
  pub fn node(&self) -> bool {
    self.node.unwrap_or(false)
  }
  pub fn electron(&self) -> bool {
    self.electron.unwrap_or(false)
  }
  pub fn nwjs(&self) -> bool {
    self.nwjs.unwrap_or(false)
  }
  pub fn electron_main(&self) -> bool {
    self.electron_main.unwrap_or(false)
  }
  pub fn electron_preload(&self) -> bool {
    self.electron_preload.unwrap_or(false)
  }
  pub fn electron_renderer(&self) -> bool {
    self.electron_renderer.unwrap_or(false)
  }
  pub fn require(&self) -> bool {
    self.require.unwrap_or(false)
  }
  pub fn node_builtins(&self) -> bool {
    self.node_builtins.unwrap_or(false)
  }
  pub fn node_prefix_for_core_modules(&self) -> bool {
    self.node_prefix_for_core_modules.unwrap_or(false)
  }
  pub fn document(&self) -> bool {
    self.document.unwrap_or(false)
  }
  pub fn import_scripts(&self) -> bool {
    self.import_scripts.unwrap_or(false)
  }
  pub fn import_scripts_in_worker(&self) -> bool {
    self.import_scripts_in_worker.unwrap_or(false)
  }
  pub fn fetch_wasm(&self) -> bool {
    self.fetch_wasm.unwrap_or(false)
  }
  pub fn global(&self) -> bool {
    self.global.unwrap_or(false)
  }
  pub fn global_this(&self) -> bool {
    self.global_this.unwrap_or(false)
  }
  pub fn big_int_literal(&self) -> bool {
    self.big_int_literal.unwrap_or(false)
  }
  pub fn r#const(&self) -> bool {
    self.r#const.unwrap_or(false)
  }
  pub fn arrow_function(&self) -> bool {
    self.arrow_function.unwrap_or(false)
  }
  pub fn for_of(&self) -> bool {
    self.for_of.unwrap_or(false)
  }
  pub fn destructuring(&self) -> bool {
    self.destructuring.unwrap_or(false)
  }
  pub fn dynamic_import(&self) -> bool {
    self.dynamic_import.unwrap_or(false)
  }
  pub fn dynamic_import_in_worker(&self) -> bool {
    self.dynamic_import_in_worker.unwrap_or(false)
  }
  pub fn module(&self) -> bool {
    self.module.unwrap_or(false)
  }
  pub fn optional_chaining(&self) -> bool {
    self.optional_chaining.unwrap_or(false)
  }
  pub fn template_literal(&self) -> bool {
    self.template_literal.unwrap_or(false)
  }
  pub fn async_function(&self) -> bool {
    self.async_function.unwrap_or(false)
  }
}

impl From<TargetProperties> for CompilerPlatform {
  fn from(value: TargetProperties) -> Self {
    Self {
      web: value.web,
      browser: value.browser,
      webworker: value.webworker,
      node: value.node,
      nwjs: value.nwjs,
      electron: value.electron,
    }
  }
}

fn version_dependent(
  major: u32,
  minor: Option<u32>,
  target_major: Option<u32>,
  target_minor: Option<u32>,
) -> bool {
  match (target_major, target_minor) {
    (Some(t_major), Some(t_minor)) => {
      t_major > major || (t_major == major && t_minor >= minor.unwrap_or(0))
    }
    (Some(t_major), None) => t_major >= major,
    _ => false,
  }
}

fn merge_target_properties(target_properties: &[TargetProperties]) -> TargetProperties {
  let mut result = TargetProperties::default();

  // Helper macro to merge a specific field across all target properties
  macro_rules! merge_field {
    ($field:ident) => {{
      let mut has_true = false;
      let mut has_false = false;

      for tp in target_properties {
        match tp.$field {
          Some(true) => has_true = true,
          Some(false) => has_false = true,
          None => {}
        }
      }

      if has_true || has_false {
        result.$field = (!(has_false && has_true)).then_some(has_true);
      }
    }};
  }

  // Merge all fields
  merge_field!(web);
  merge_field!(browser);
  merge_field!(webworker);
  merge_field!(node);
  merge_field!(electron);
  merge_field!(nwjs);
  merge_field!(electron_main);
  merge_field!(electron_preload);
  merge_field!(electron_renderer);
  merge_field!(require);
  merge_field!(node_builtins);
  merge_field!(node_prefix_for_core_modules);
  merge_field!(document);
  merge_field!(import_scripts);
  merge_field!(import_scripts_in_worker);
  merge_field!(fetch_wasm);
  merge_field!(global);
  merge_field!(global_this);
  merge_field!(big_int_literal);
  merge_field!(r#const);
  merge_field!(arrow_function);
  merge_field!(for_of);
  merge_field!(destructuring);
  merge_field!(dynamic_import);
  merge_field!(dynamic_import_in_worker);
  merge_field!(module);
  merge_field!(optional_chaining);
  merge_field!(template_literal);
  merge_field!(async_function);

  result
}

fn get_target_properties(target: &str, context: &Context) -> TargetProperties {
  // Parse target string
  if let Some(captures) = regex::Regex::new(r"^(async-)?node((\d+)(?:\.(\d+))?)?$")
    .expect("should initialize `Regex`")
    .captures(target)
  {
    let async_flag = captures.get(1).is_some();
    let major = captures.get(3).map(|m| {
      m.as_str()
        .parse::<u32>()
        .expect("should initialize `Regex`")
    });
    let minor = captures.get(4).map(|m| {
      m.as_str()
        .parse::<u32>()
        .expect("should initialize `Regex`")
    });

    return TargetProperties {
      node: Some(true),
      electron: Some(false),
      nwjs: Some(false),
      web: Some(false),
      webworker: Some(false),
      browser: Some(false),
      require: Some(!async_flag),
      node_builtins: Some(true),
      node_prefix_for_core_modules: Some(match major {
        Some(m) if m < 15 => version_dependent(14, Some(18), major, minor),
        Some(_) => version_dependent(16, None, major, minor),
        None => false,
      }),
      global: Some(true),
      document: Some(false),
      fetch_wasm: Some(false),
      import_scripts: Some(false),
      import_scripts_in_worker: Some(false),
      global_this: Some(version_dependent(12, None, major, minor)),
      r#const: Some(version_dependent(6, None, major, minor)),
      template_literal: Some(version_dependent(4, None, major, minor)),
      optional_chaining: Some(version_dependent(14, None, major, minor)),
      arrow_function: Some(version_dependent(6, None, major, minor)),
      async_function: Some(version_dependent(7, Some(6), major, minor)),
      for_of: Some(version_dependent(5, None, major, minor)),
      destructuring: Some(version_dependent(6, None, major, minor)),
      big_int_literal: Some(version_dependent(10, Some(4), major, minor)),
      dynamic_import: Some(version_dependent(12, Some(17), major, minor)),
      dynamic_import_in_worker: if major.is_some() { Some(false) } else { None },
      module: Some(version_dependent(12, Some(17), major, minor)),
      ..Default::default()
    };
  }

  if let Some(captures) = regex::Regex::new(r"^browserslist(?::(.+))?$")
    .expect("should initialize `Regex`")
    .captures(target)
  {
    let rest = captures.get(1).map(|m| m.as_str());

    let browsers = load_browserslist(rest.map(|r| r.trim()), context).unwrap_or_default();

    if browsers.is_empty() {
      panic!("No browserslist config found for target '{target}'. Please configure browserslist.");
    }

    return super::browserslist_target::resolve(browsers);
  }

  // Handle web target
  if target == "web" {
    return TargetProperties {
      web: Some(true),
      browser: Some(true),
      webworker: None,
      node: Some(false),
      electron: Some(false),
      nwjs: Some(false),
      document: Some(true),
      import_scripts_in_worker: Some(true),
      fetch_wasm: Some(true),
      node_builtins: Some(false),
      import_scripts: Some(false),
      require: Some(false),
      global: Some(false),
      ..Default::default()
    };
  }

  // Webworker target
  if target == "webworker" {
    return TargetProperties {
      web: Some(true),
      browser: Some(true),
      webworker: Some(true),
      node: Some(false),
      electron: Some(false),
      nwjs: Some(false),
      import_scripts: Some(true),
      import_scripts_in_worker: Some(true),
      fetch_wasm: Some(true),
      node_builtins: Some(false),
      require: Some(false),
      document: Some(false),
      global: Some(false),
      ..Default::default()
    };
  }

  // Electron target
  if let Some(captures) =
    regex::Regex::new(r"^electron((\d+)(?:\.(\d+))?)?-(main|preload|renderer)$")
      .expect("should initialize `Regex`")
      .captures(target)
  {
    let major = captures.get(2).map(|m| {
      m.as_str()
        .parse::<u32>()
        .expect("should initialize `Regex`")
    });
    let minor = captures.get(3).map(|m| {
      m.as_str()
        .parse::<u32>()
        .expect("should initialize `Regex`")
    });
    let context = captures
      .get(4)
      .map(|m| m.as_str())
      .expect("should initialize `Regex`");

    return TargetProperties {
      node: Some(true),
      electron: Some(true),
      web: Some(context != "main"),
      webworker: Some(false),
      browser: Some(false),
      nwjs: Some(false),
      electron_main: Some(context == "main"),
      electron_preload: Some(context == "preload"),
      electron_renderer: Some(context == "renderer"),
      global: Some(true),
      node_builtins: Some(true),
      node_prefix_for_core_modules: Some(version_dependent(15, None, major, minor)),
      require: Some(true),
      document: Some(context == "renderer"),
      fetch_wasm: Some(context == "renderer"),
      import_scripts: Some(false),
      import_scripts_in_worker: Some(true),
      global_this: Some(version_dependent(5, None, major, minor)),
      r#const: Some(version_dependent(1, Some(1), major, minor)),
      template_literal: Some(version_dependent(1, Some(1), major, minor)),
      optional_chaining: Some(version_dependent(8, None, major, minor)),
      arrow_function: Some(version_dependent(1, Some(1), major, minor)),
      async_function: Some(version_dependent(1, Some(7), major, minor)),
      for_of: Some(version_dependent(0, Some(36), major, minor)),
      destructuring: Some(version_dependent(1, Some(1), major, minor)),
      big_int_literal: Some(version_dependent(4, None, major, minor)),
      dynamic_import: Some(version_dependent(11, None, major, minor)),
      dynamic_import_in_worker: if major.is_some() { Some(false) } else { None },
      module: Some(version_dependent(11, None, major, minor)),
    };
  }

  // NW.js target
  if let Some(captures) = regex::Regex::new(r"^(?:nwjs|node-webkit)((\d+)(?:\.(\d+))?)?$")
    .expect("should initialize `Regex`")
    .captures(target)
  {
    let major = captures.get(2).map(|m| {
      m.as_str()
        .parse::<u32>()
        .expect("should initialize `Regex`")
    });
    let minor = captures.get(3).map(|m| {
      m.as_str()
        .parse::<u32>()
        .expect("should initialize `Regex`")
    });

    return TargetProperties {
      node: Some(true),
      web: Some(true),
      nwjs: Some(true),
      webworker: None,
      browser: Some(false),
      electron: Some(false),
      global: Some(true),
      node_builtins: Some(true),
      document: Some(false),
      import_scripts_in_worker: Some(false),
      fetch_wasm: Some(false),
      import_scripts: Some(false),
      require: Some(false),
      global_this: Some(version_dependent(0, Some(43), major, minor)),
      r#const: Some(version_dependent(0, Some(15), major, minor)),
      template_literal: Some(version_dependent(0, Some(13), major, minor)),
      optional_chaining: Some(version_dependent(0, Some(44), major, minor)),
      arrow_function: Some(version_dependent(0, Some(15), major, minor)),
      async_function: Some(version_dependent(0, Some(21), major, minor)),
      for_of: Some(version_dependent(0, Some(13), major, minor)),
      destructuring: Some(version_dependent(0, Some(15), major, minor)),
      big_int_literal: Some(version_dependent(0, Some(32), major, minor)),
      dynamic_import: Some(version_dependent(0, Some(43), major, minor)),
      dynamic_import_in_worker: if major.is_some() { Some(false) } else { None },
      module: Some(version_dependent(0, Some(43), major, minor)),
      ..Default::default()
    };
  }

  // ES version target
  if let Some(captures) = regex::Regex::new(r"^es(\d+)$")
    .expect("should initialize `Regex`")
    .captures(target)
  {
    let mut version = captures
      .get(1)
      .expect("should initialize `Regex`")
      .as_str()
      .parse::<u32>()
      .expect("should initialize `Regex`");
    if version < 1000 {
      version += 2009;
    }

    return TargetProperties {
      r#const: Some(version >= 2015),
      template_literal: Some(version >= 2015),
      optional_chaining: Some(version >= 2020),
      arrow_function: Some(version >= 2015),
      for_of: Some(version >= 2015),
      destructuring: Some(version >= 2015),
      module: Some(version >= 2015),
      async_function: Some(version >= 2017),
      global_this: Some(version >= 2020),
      big_int_literal: Some(version >= 2020),
      dynamic_import: Some(version >= 2020),
      dynamic_import_in_worker: Some(version >= 2020),
      ..Default::default()
    };
  }

  panic!("Unknown target {target}");
}

pub fn get_targets_properties(targets: &[String], context: &Context) -> TargetProperties {
  merge_target_properties(
    &targets
      .iter()
      .map(|t| get_target_properties(t, context))
      .collect::<Vec<_>>(),
  )
}
