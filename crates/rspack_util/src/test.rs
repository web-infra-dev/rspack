use once_cell::sync::Lazy;

static RSPACK_HOT_TEST: Lazy<String> =
  Lazy::new(|| std::env::var("RSPACK_HOT_TEST").ok().unwrap_or_default());

pub fn is_hot_test() -> bool {
  *RSPACK_HOT_TEST == "true"
}

pub static HOT_TEST_DEFINE_GLOBAL: Lazy<String> = Lazy::new(|| {
  is_hot_test()
    .then(|| {
      r#"
      self.__HMR_UPDATED_RUNTIME__ = {
        javascript: {
          outdatedModules: [],
          outdatedDependencies: [],

          acceptedModules: [],
          updatedModules: [],
          updatedRuntime: [],
          disposedModules: [],
        },
        statusPath: []
      };
      "#
      .to_string()
    })
    .unwrap_or_default()
});

pub static HOT_TEST_STATUS_CHANGE: Lazy<String> = Lazy::new(|| {
  is_hot_test()
    .then(|| {
      r#"
      self.__HMR_UPDATED_RUNTIME__.statusPath.push(newStatus);
      "#
      .to_string()
    })
    .unwrap_or_default()
});

pub static HOT_TEST_OUTDATED: Lazy<String> = Lazy::new(|| {
  is_hot_test()
    .then(|| {
      r#"
      self.__HMR_UPDATED_RUNTIME__.javascript.outdatedModules = outdatedModules;
	    self.__HMR_UPDATED_RUNTIME__.javascript.outdatedDependencies = outdatedDependencies;
      "#
      .to_string()
    })
    .unwrap_or_default()
});

pub static HOT_TEST_DISPOSE: Lazy<String> = Lazy::new(|| {
  is_hot_test()
    .then(|| {
      r#"
      if (disposeHandlers.length > 0) {
        self.__HMR_UPDATED_RUNTIME__.javascript.disposedModules.push(moduleId);
      }
      "#
      .to_string()
    })
    .unwrap_or_default()
});

pub static HOT_TEST_UPDATED: Lazy<String> = Lazy::new(|| {
  is_hot_test()
    .then(|| {
      r#"
      self.__HMR_UPDATED_RUNTIME__.javascript.updatedModules.push(updateModuleId);
      "#
      .to_string()
    })
    .unwrap_or_default()
});

pub static HOT_TEST_RUNTIME: Lazy<String> = Lazy::new(|| {
  is_hot_test()
    .then(|| {
      r#"
      currentUpdateRuntime[i](new Proxy(__webpack_require__, {
        set(target, prop, value, receiver) {
          self.__HMR_UPDATED_RUNTIME__.javascript.updatedRuntime.push(`__webpack_require__.${prop}`);
          return Reflect.set(target, prop, value, receiver);
        }
      }));
      "#
      .to_string()
    })
    .unwrap_or_else(|| "currentUpdateRuntime[i](__webpack_require__);".to_string())
});

pub static HOT_TEST_ACCEPT: Lazy<String> = Lazy::new(|| {
  is_hot_test()
    .then(|| {
      r#"
      self.__HMR_UPDATED_RUNTIME__.javascript.acceptedModules.push(dependency);
      "#
      .to_string()
    })
    .unwrap_or_default()
});
