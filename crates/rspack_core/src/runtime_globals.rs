use bitflags::bitflags;

use crate::CompilerOptions;

#[rspack_cacheable::cacheable]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct RuntimeGlobals(u128);

macro_rules! define_runtime_globals {
  (@step ($val:expr) ({$($acc:tt)*}) ($(#[$($attr:tt)+])* const $name:ident; $($rest:tt)*)) => {
    define_runtime_globals! {
      @step ($val << 1) ({
        $($acc)*
        $(#[$($attr)+])*
        const $name = $val;
      }) ($($rest)*)
    }
  };
  (@step ($val:expr) ({$($acc:tt)*}) ()) => {
    bitflags! {
      impl RuntimeGlobals: u128 {
        $($acc)*
      }
    }
  };
  ($($rest:tt)*) => {
    define_runtime_globals! {
      @step (1u128) ({}) ($($rest)*)
    }
  };
}

define_runtime_globals! {
  const REQUIRE_SCOPE;

  /**
   * the internal module object
   */
  const MODULE;

  /**
   * the internal module object
   */
  const MODULE_ID;

  /**
   * the internal require function
   */
  const REQUIRE;

  /**
   * the module cache
   */
  const MODULE_CACHE;

  /**
   * the chunk ensure function
   */
  const ENSURE_CHUNK;

  /**
   * an object with handlers to ensure a chunk
   */
  const ENSURE_CHUNK_HANDLERS;

  /**
   * the bundle public path
   */
  const PUBLIC_PATH;

  /**
   * the filename of the script part of the chunk
   */
  const GET_CHUNK_SCRIPT_FILENAME;

  /**
   * the filename of the css part of the chunk
   */
  const GET_CHUNK_CSS_FILENAME;

  /**
   * function to load a script tag.
   * Arguments: (url: string, done: (event) => void), key?: string | number, chunkId?: string | number) => void
   * done function is called when loading has finished or timeout occurred.
   * It will attach to existing script tags with data-webpack == uniqueName + ":" + key or src == url.
   */
  const LOAD_SCRIPT;

  /**
   * the shorthand for Object.prototype.hasOwnProperty
   * using of it decreases the compiled bundle size
   */
  const HAS_OWN_PROPERTY;

  /**
   * the module functions, with only write access
   */
  const MODULE_FACTORIES_ADD_ONLY;

  /**
   * register deferred code, which will run when certain
   * chunks are loaded.
   * Signature: (chunkIds: Id[], fn: () => any, priority: int >= 0 = 0) => any
   * Returned value will be returned directly when all chunks are already loaded
   * When (priority & 1) it will wait for all other handlers with lower priority to
   * be executed before itself is executed
   */
  const ON_CHUNKS_LOADED;

  /**
   * global callback functions for installing chunks
   */
  const CHUNK_CALLBACK;

  /**
   * the module functions
   */
  const MODULE_FACTORIES;

  /**
   * interceptor for module executions
   */
  const INTERCEPT_MODULE_EXECUTION;

  /**
   * function downloading the update manifest
   */
  const HMR_DOWNLOAD_MANIFEST;

  /**
   * array with handler functions to download chunk updates
   */
  const HMR_DOWNLOAD_UPDATE_HANDLERS;

  const HMR_INVALIDATE_MODULE_HANDLERS;

  /**
   * the filename of the HMR manifest
   */
  const GET_UPDATE_MANIFEST_FILENAME;

  /**
   * the filename of the script part of the hot update chunk
   */
  const GET_CHUNK_UPDATE_SCRIPT_FILENAME;

  /**
   * the filename of the css part of the hot update chunk
   */
  const GET_CHUNK_UPDATE_CSS_FILENAME;

  /**
   * object with all hmr module data for all modules
   */
  const HMR_MODULE_DATA;

  /**
   * the prefix for storing state of runtime modules when hmr is enabled
   */
  const HMR_RUNTIME_STATE_PREFIX;

  /**
   * method to install a chunk that was loaded somehow
   * Signature: ({ id, ids, modules, runtime }) => void
   */
  const EXTERNAL_INSTALL_CHUNK;

  /**
   * the webpack hash
   */
  const GET_FULL_HASH;

  /**
   * the global object
   */
  const GLOBAL;

  /**
   * runtime need to return the exports of the last entry module
   */
  const RETURN_EXPORTS_FROM_RUNTIME;

  /**
   * instantiate a wasm instance from module exports object, id, hash and importsObject
   */
  const INSTANTIATE_WASM;

  /**
   * Creates an async module. The body function must be a async function.
   * "module.exports" will be decorated with an AsyncModulePromise.
   * The body function will be called.
   * To handle async dependencies correctly do this: "([a, b, c] = await handleDependencies([a, b, c]));".
   * If "hasAwaitAfterDependencies" is truthy, "handleDependencies()" must be called at the end of the body function.
   * Signature: function(
   * module: Module,
   * body: (handleDependencies: (deps: AsyncModulePromise[]) => Promise<any[]> & () => void,
   * hasAwaitAfterDependencies?: boolean
   * ) => void
   */
  const ASYNC_MODULE;

  /**
   * the baseURI of current document
   */
  const BASE_URI;

  const MODULE_LOADED;

  const STARTUP_ENTRYPOINT;
  const STARTUP_CHUNK_DEPENDENCIES;

  const CREATE_SCRIPT_URL;

  const CREATE_SCRIPT;

  const GET_TRUSTED_TYPES_POLICY;

  const DEFINE_PROPERTY_GETTERS;

  const ENTRY_MODULE_ID;

  const STARTUP_NO_DEFAULT;

  const ENSURE_CHUNK_INCLUDE_ENTRIES;

  const STARTUP;

  const MAKE_NAMESPACE_OBJECT;

  const EXPORTS;

  const COMPAT_GET_DEFAULT_EXPORT;

  const CREATE_FAKE_NAMESPACE_OBJECT;

  const NODE_MODULE_DECORATOR;

  const ESM_MODULE_DECORATOR;

  /**
   * the System.register context object
   */
  const SYSTEM_CONTEXT;

  const THIS_AS_EXPORTS;

  const CURRENT_REMOTE_GET_SCOPE;

  const SHARE_SCOPE_MAP;

  const INITIALIZE_SHARING;

  const SCRIPT_NONCE;

  const RELATIVE_URL;

  const CHUNK_NAME;

  const RUNTIME_ID;

  // prefetch and preload
  const PREFETCH_CHUNK;

  const PREFETCH_CHUNK_HANDLERS;

  const PRELOAD_CHUNK;

  const PRELOAD_CHUNK_HANDLERS;

  const UNCAUGHT_ERROR_HANDLER;

  // rspack only
  const RSPACK_VERSION;

  const HAS_CSS_MODULES;

  // rspack only
  const RSPACK_UNIQUE_ID;

  const HAS_FETCH_PRIORITY;

  // amd module support
  const AMD_DEFINE;
  const AMD_OPTIONS;

  const TO_BINARY;

  // defer import support
  const ASYNC_MODULE_EXPORT_SYMBOL;
  const MAKE_DEFERRED_NAMESPACE_OBJECT;
  const MAKE_DEFERRED_NAMESPACE_OBJECT_SYMBOL;
  const MAKE_OPTIMIZED_DEFERRED_NAMESPACE_OBJECT;

  // rspack only
  const ASYNC_STARTUP;

  // react server component
  const RSC_MANIFEST;
}

impl Default for RuntimeGlobals {
  fn default() -> Self {
    Self::empty()
  }
}

pub fn runtime_globals_to_string(
  runtime_globals: &RuntimeGlobals,
  compiler_options: &CompilerOptions,
) -> String {
  let scope_name = runtime_variable_to_string(&RuntimeVariable::Require, compiler_options);
  match *runtime_globals {
    RuntimeGlobals::REQUIRE_SCOPE => format!("{scope_name}.*"),
    RuntimeGlobals::MODULE => "module".to_string(),
    RuntimeGlobals::MODULE_ID => "module.id".to_string(),
    RuntimeGlobals::MODULE_LOADED => "module.loaded".to_string(),
    RuntimeGlobals::REQUIRE => scope_name,
    RuntimeGlobals::MODULE_CACHE => format!("{scope_name}.c"),
    RuntimeGlobals::ENSURE_CHUNK => format!("{scope_name}.e"),
    RuntimeGlobals::ENSURE_CHUNK_HANDLERS => format!("{scope_name}.f"),
    RuntimeGlobals::PUBLIC_PATH => format!("{scope_name}.p"),
    RuntimeGlobals::GET_CHUNK_SCRIPT_FILENAME => format!("{scope_name}.u"),
    RuntimeGlobals::GET_CHUNK_CSS_FILENAME => format!("{scope_name}.k"),
    RuntimeGlobals::LOAD_SCRIPT => format!("{scope_name}.l"),
    RuntimeGlobals::HAS_OWN_PROPERTY => format!("{scope_name}.o"),
    RuntimeGlobals::MODULE_FACTORIES_ADD_ONLY => format!("{scope_name}.m (add only)"),
    RuntimeGlobals::ON_CHUNKS_LOADED => format!("{scope_name}.O"),
    RuntimeGlobals::CHUNK_CALLBACK => "global chunk callback".to_string(),
    RuntimeGlobals::MODULE_FACTORIES => format!("{scope_name}.m"),
    RuntimeGlobals::INTERCEPT_MODULE_EXECUTION => format!("{scope_name}.i"),
    RuntimeGlobals::HMR_DOWNLOAD_MANIFEST => format!("{scope_name}.hmrM"),
    RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS => format!("{scope_name}.hmrC"),
    RuntimeGlobals::HMR_INVALIDATE_MODULE_HANDLERS => format!("{scope_name}.hmrI"),
    RuntimeGlobals::HMR_MODULE_DATA => format!("{scope_name}.hmrD"),
    RuntimeGlobals::HMR_RUNTIME_STATE_PREFIX => format!("{scope_name}.hmrS"),
    RuntimeGlobals::GET_UPDATE_MANIFEST_FILENAME => format!("{scope_name}.hmrF"),
    RuntimeGlobals::GET_CHUNK_UPDATE_SCRIPT_FILENAME => format!("{scope_name}.hu"),
    RuntimeGlobals::GET_CHUNK_UPDATE_CSS_FILENAME => format!("{scope_name}.hk"),
    RuntimeGlobals::AMD_DEFINE => format!("{scope_name}.amdD"),
    RuntimeGlobals::AMD_OPTIONS => format!("{scope_name}.amdO"),
    RuntimeGlobals::EXTERNAL_INSTALL_CHUNK => format!("{scope_name}.C"),
    RuntimeGlobals::GET_FULL_HASH => format!("{scope_name}.h"),
    RuntimeGlobals::GLOBAL => format!("{scope_name}.g"),
    RuntimeGlobals::RETURN_EXPORTS_FROM_RUNTIME => "return-exports-from-runtime".to_string(),
    RuntimeGlobals::INSTANTIATE_WASM => format!("{scope_name}.v"),
    RuntimeGlobals::ASYNC_MODULE => format!("{scope_name}.a"),
    RuntimeGlobals::ASYNC_MODULE_EXPORT_SYMBOL => format!("{scope_name}.aE"),
    RuntimeGlobals::BASE_URI => format!("{scope_name}.b"),
    RuntimeGlobals::STARTUP_ENTRYPOINT => format!("{scope_name}.X"),
    RuntimeGlobals::STARTUP_CHUNK_DEPENDENCIES => format!("{scope_name}.x (chunk dependencies)"),
    RuntimeGlobals::CREATE_SCRIPT_URL => format!("{scope_name}.tu"),
    RuntimeGlobals::CREATE_SCRIPT => format!("{scope_name}.ts"),
    RuntimeGlobals::GET_TRUSTED_TYPES_POLICY => format!("{scope_name}.tt"),
    RuntimeGlobals::DEFINE_PROPERTY_GETTERS => format!("{scope_name}.d"),
    RuntimeGlobals::ENTRY_MODULE_ID => format!("{scope_name}.s"),
    RuntimeGlobals::STARTUP_NO_DEFAULT => format!("{scope_name}.x (no default handler)"),
    RuntimeGlobals::ENSURE_CHUNK_INCLUDE_ENTRIES => format!("{scope_name}.f (include entries)"),
    RuntimeGlobals::STARTUP => format!("{scope_name}.x"),
    RuntimeGlobals::MAKE_NAMESPACE_OBJECT => format!("{scope_name}.r"),
    RuntimeGlobals::MAKE_DEFERRED_NAMESPACE_OBJECT => format!("{scope_name}.z"),
    RuntimeGlobals::MAKE_DEFERRED_NAMESPACE_OBJECT_SYMBOL => format!("{scope_name}.zS"),
    RuntimeGlobals::MAKE_OPTIMIZED_DEFERRED_NAMESPACE_OBJECT => format!("{scope_name}.zO"),
    RuntimeGlobals::EXPORTS => {
      runtime_variable_to_string(&RuntimeVariable::Exports, compiler_options)
    }
    RuntimeGlobals::COMPAT_GET_DEFAULT_EXPORT => format!("{scope_name}.n"),
    RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT => format!("{scope_name}.t"),
    RuntimeGlobals::ESM_MODULE_DECORATOR => format!("{scope_name}.hmd"),
    RuntimeGlobals::NODE_MODULE_DECORATOR => format!("{scope_name}.nmd"),
    RuntimeGlobals::SYSTEM_CONTEXT => format!("{scope_name}.y"),
    RuntimeGlobals::THIS_AS_EXPORTS => "top-level-this-exports".to_string(),
    RuntimeGlobals::CURRENT_REMOTE_GET_SCOPE => format!("{scope_name}.R"),
    RuntimeGlobals::SHARE_SCOPE_MAP => format!("{scope_name}.S"),
    RuntimeGlobals::INITIALIZE_SHARING => format!("{scope_name}.I"),
    RuntimeGlobals::SCRIPT_NONCE => format!("{scope_name}.nc"),
    RuntimeGlobals::RELATIVE_URL => format!("{scope_name}.U"),
    RuntimeGlobals::CHUNK_NAME => format!("{scope_name}.cn"),
    RuntimeGlobals::RUNTIME_ID => format!("{scope_name}.j"),
    RuntimeGlobals::PREFETCH_CHUNK => format!("{scope_name}.E"),
    RuntimeGlobals::PREFETCH_CHUNK_HANDLERS => format!("{scope_name}.F"),
    RuntimeGlobals::PRELOAD_CHUNK => format!("{scope_name}.G"),
    RuntimeGlobals::PRELOAD_CHUNK_HANDLERS => format!("{scope_name}.H"),
    RuntimeGlobals::UNCAUGHT_ERROR_HANDLER => format!("{scope_name}.oe"),
    // rspack only
    RuntimeGlobals::RSPACK_VERSION => format!("{scope_name}.rv"),
    RuntimeGlobals::RSPACK_UNIQUE_ID => format!("{scope_name}.ruid"),
    RuntimeGlobals::HAS_CSS_MODULES => "has css modules".to_string(),
    RuntimeGlobals::ASYNC_STARTUP => format!("{scope_name}.asyncStartup"),
    RuntimeGlobals::HAS_FETCH_PRIORITY => "has fetch priority".to_string(),

    RuntimeGlobals::RSC_MANIFEST => format!("{scope_name}.rscM"),
    RuntimeGlobals::TO_BINARY => format!("{scope_name}.tb"),
    _ => unreachable!(),
  }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum RuntimeVariable {
  Require,
  Modules,
  ModuleCache,
  Module,
  Exports,
  StartupExec,
}

pub fn runtime_variable_to_string(
  runtime_variable: &RuntimeVariable,
  _compiler_options: &CompilerOptions,
) -> String {
  // TODO: use compiler options to get runtime variable names
  match *runtime_variable {
    RuntimeVariable::Require => "__webpack_require__".to_string(),
    RuntimeVariable::Modules => "__webpack_modules__".to_string(),
    RuntimeVariable::ModuleCache => "__webpack_module_cache__".to_string(),
    RuntimeVariable::Exports => "__webpack_exports__".to_string(),
    RuntimeVariable::Module => "__webpack_module__".to_string(),
    RuntimeVariable::StartupExec => "__webpack_exec__".to_string(),
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_iter() {
    let flags = RuntimeGlobals::PUBLIC_PATH | RuntimeGlobals::GET_CHUNK_CSS_FILENAME;
    let flags: Vec<_> = flags.iter().collect();
    assert_eq!(flags.len(), 2);
    assert_eq!(flags[0], RuntimeGlobals::PUBLIC_PATH);
    assert_eq!(flags[1], RuntimeGlobals::GET_CHUNK_CSS_FILENAME);
  }
}
