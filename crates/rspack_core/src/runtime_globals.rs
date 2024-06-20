use std::fmt;

use bitflags::bitflags;
use swc_core::ecma::atoms::Atom;

bitflags! {
  #[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
  pub struct RuntimeGlobals: u128 {
    const REQUIRE_SCOPE = 1 << 0;

    /**
     * the internal module object
     */
    const MODULE = 1 << 3;

    /**
     * the internal module object
     */
    const MODULE_ID = 1 << 4;

    /**
     * the internal require function
     */
    const REQUIRE = 1 << 5;

    /**
     * the module cache
     */
    const MODULE_CACHE = 1 << 6;

    /**
     * the chunk ensure function
     */
    const ENSURE_CHUNK = 1 << 7;

    /**
     * an object with handlers to ensure a chunk
     */
    const ENSURE_CHUNK_HANDLERS = 1 << 8;

    /**
     * the bundle public path
     */
    const PUBLIC_PATH = 1 << 9;

    /**
     * the filename of the script part of the chunk
     */
    const GET_CHUNK_SCRIPT_FILENAME = 1 << 10;

    /**
     * the filename of the css part of the chunk
     */
    const GET_CHUNK_CSS_FILENAME = 1 << 11;

    /**
     * function to load a script tag.
     * Arguments: (url: string, done: (event) => void), key?: string | number, chunkId?: string | number) => void
     * done function is called when loading has finished or timeout occurred.
     * It will attach to existing script tags with data-webpack == uniqueName + ":" + key or src == url.
     */
    const LOAD_SCRIPT = 1 << 12;

    /**
     * the shorthand for Object.prototype.hasOwnProperty
     * using of it decreases the compiled bundle size
     */
    const HAS_OWN_PROPERTY = 1 << 13;

    /**
     * the module functions, with only write access
     */
    const MODULE_FACTORIES_ADD_ONLY = 1 << 14;

    /**
     * register deferred code, which will run when certain
     * chunks are loaded.
     * Signature: (chunkIds: Id[], fn: () => any, priority: int >= 0 = 0) => any
     * Returned value will be returned directly when all chunks are already loaded
     * When (priority & 1) it will wait for all other handlers with lower priority to
     * be executed before itself is executed
     */
    const ON_CHUNKS_LOADED = 1 << 15;

    /**
     * global callback functions for installing chunks
     */
    const CHUNK_CALLBACK = 1 << 16;

    /**
     * the module functions
     */
    const MODULE_FACTORIES = 1 << 17;

    /**
     * interceptor for module executions
     */
    const INTERCEPT_MODULE_EXECUTION = 1 << 18;

    /**
     * function downloading the update manifest
     */
    const HMR_DOWNLOAD_MANIFEST = 1 << 19;

    /**
     * array with handler functions to download chunk updates
     */
    const HMR_DOWNLOAD_UPDATE_HANDLERS = 1 << 20;

    /**
     * the filename of the HMR manifest
     */
    const GET_UPDATE_MANIFEST_FILENAME = 1 << 21;

    /**
     * the filename of the script part of the hot update chunk
     */
    const GET_CHUNK_UPDATE_SCRIPT_FILENAME = 1 << 22;

    /**
     * the filename of the css part of the hot update chunk
     */
    const GET_CHUNK_UPDATE_CSS_FILENAME = 1 << 23;

    /**
     * object with all hmr module data for all modules
     */
    const HMR_MODULE_DATA = 1 << 24;

    /**
     * the prefix for storing state of runtime modules when hmr is enabled
     */
    const HMR_RUNTIME_STATE_PREFIX = 1 << 25;

    /**
     * method to install a chunk that was loaded somehow
     * Signature: ({ id, ids, modules, runtime }) => void
     */
    const EXTERNAL_INSTALL_CHUNK = 1 << 26;

    /**
     * the webpack hash
     */
    const GET_FULL_HASH = 1 << 27;

    /**
     * the global object
     */
    const GLOBAL = 1 << 28;

    /**
     * runtime need to return the exports of the last entry module
     */
    const RETURN_EXPORTS_FROM_RUNTIME = 1 << 29;

    /**
     * instantiate a wasm instance from module exports object, id, hash and importsObject
     */
    const INSTANTIATE_WASM = 1 << 30;

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
    const ASYNC_MODULE = 1 << 31;

    /**
     * the baseURI of current document
     */
    const BASE_URI = 1 << 32;

    const MODULE_LOADED = 1 << 33;

    const STARTUP_ENTRYPOINT = 1 << 34;

    const CREATE_SCRIPT_URL = 1 << 35;

    const CREATE_SCRIPT = 1 << 36;

    const GET_TRUSTED_TYPES_POLICY = 1 << 37;

    const DEFINE_PROPERTY_GETTERS = 1 << 38;

    const ENTRY_MODULE_ID = 1 << 39;

    const STARTUP_NO_DEFAULT = 1 << 40;

    const ENSURE_CHUNK_INCLUDE_ENTRIES = 1 << 41;

    const STARTUP = 1 << 42;

    const MAKE_NAMESPACE_OBJECT = 1 << 43;

    const EXPORTS = 1 << 44;

    const COMPAT_GET_DEFAULT_EXPORT = 1 << 45;

    const CREATE_FAKE_NAMESPACE_OBJECT = 1 << 46;

    const NODE_MODULE_DECORATOR = 1 << 47;

    const HARMONY_MODULE_DECORATOR = 1 << 48;

    /**
     * the System.register context object
     */
    const SYSTEM_CONTEXT = 1 << 49;

    const THIS_AS_EXPORTS = 1 << 50;

    const CURRENT_REMOTE_GET_SCOPE = 1 << 51;

    const SHARE_SCOPE_MAP = 1 << 52;

    const INITIALIZE_SHARING = 1 << 53;

    const SCRIPT_NONCE = 1 << 54;

    const RELATIVE_URL = 1 << 55;

    const CHUNK_NAME = 1 << 56;

    const RUNTIME_ID = 1 << 57;

    // prefetch and preload
    const PREFETCH_CHUNK = 1 << 58;

    const PREFETCH_CHUNK_HANDLERS = 1 << 59;

    const PRELOAD_CHUNK = 1 << 60;

    const PRELOAD_CHUNK_HANDLERS = 1 << 61;

    // rspack only
    const RSPACK_VERSION = 1 << 62;

    const HAS_CSS_MODULES = 1 << 63;

    const RSPACK_UNIQUE_ID = 1 << 64;
  }
}

impl fmt::Display for RuntimeGlobals {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let name = self.name();
    f.write_str(name)
  }
}

impl Default for RuntimeGlobals {
  fn default() -> Self {
    Self::empty()
  }
}

impl RuntimeGlobals {
  pub const fn name(&self) -> &'static str {
    use RuntimeGlobals as R;
    match *self {
      R::REQUIRE_SCOPE => "__webpack_require__.*",
      R::MODULE => "module",
      R::MODULE_ID => "module.id",
      R::MODULE_LOADED => "module.loaded",
      R::REQUIRE => "__webpack_require__",
      R::MODULE_CACHE => "__webpack_require__.c",
      R::ENSURE_CHUNK => "__webpack_require__.e",
      R::ENSURE_CHUNK_HANDLERS => "__webpack_require__.f",
      R::PUBLIC_PATH => "__webpack_require__.p",
      R::GET_CHUNK_SCRIPT_FILENAME => "__webpack_require__.u",
      R::GET_CHUNK_CSS_FILENAME => "__webpack_require__.k",
      R::LOAD_SCRIPT => "__webpack_require__.l",
      R::HAS_OWN_PROPERTY => "__webpack_require__.o",
      R::MODULE_FACTORIES_ADD_ONLY => "__webpack_require__.m (add only)",
      R::ON_CHUNKS_LOADED => "__webpack_require__.O",
      R::CHUNK_CALLBACK => "webpackChunk",
      R::MODULE_FACTORIES => "__webpack_require__.m",
      R::INTERCEPT_MODULE_EXECUTION => "__webpack_require__.i",
      R::HMR_DOWNLOAD_MANIFEST => "__webpack_require__.hmrM",
      R::HMR_DOWNLOAD_UPDATE_HANDLERS => "__webpack_require__.hmrC",
      R::GET_UPDATE_MANIFEST_FILENAME => "__webpack_require__.hmrF",
      R::GET_CHUNK_UPDATE_SCRIPT_FILENAME => "__webpack_require__.hu",
      R::GET_CHUNK_UPDATE_CSS_FILENAME => "__webpack_require__.hk",
      R::HMR_MODULE_DATA => "__webpack_require__.hmrD",
      R::HMR_RUNTIME_STATE_PREFIX => "__webpack_require__.hmrS",
      R::EXTERNAL_INSTALL_CHUNK => "__webpack_require__.C",
      R::GET_FULL_HASH => "__webpack_require__.h",
      R::GLOBAL => "__webpack_require__.g",
      R::RETURN_EXPORTS_FROM_RUNTIME => "return-exports-from-runtime",
      R::INSTANTIATE_WASM => "__webpack_require__.v",
      R::ASYNC_MODULE => "__webpack_require__.a",
      R::BASE_URI => "__webpack_require__.b",
      R::STARTUP_ENTRYPOINT => "__webpack_require__.X",
      R::CREATE_SCRIPT_URL => "__webpack_require__.tu",
      R::CREATE_SCRIPT => "__webpack_require__.ts",
      R::GET_TRUSTED_TYPES_POLICY => "__webpack_require__.tt",
      R::DEFINE_PROPERTY_GETTERS => "__webpack_require__.d",
      R::ENTRY_MODULE_ID => "__webpack_require__.s",
      R::STARTUP_NO_DEFAULT => "__webpack_require__.x (no default handler)",
      R::ENSURE_CHUNK_INCLUDE_ENTRIES => "__webpack_require__.f (include entries)",
      R::STARTUP => "__webpack_require__.x",
      R::MAKE_NAMESPACE_OBJECT => "__webpack_require__.r",
      R::EXPORTS => "__webpack_exports__",
      R::COMPAT_GET_DEFAULT_EXPORT => "__webpack_require__.n",
      R::CREATE_FAKE_NAMESPACE_OBJECT => "__webpack_require__.t",
      R::HARMONY_MODULE_DECORATOR => "__webpack_require__.hmd",
      R::NODE_MODULE_DECORATOR => "__webpack_require__.nmd",
      R::SYSTEM_CONTEXT => "__webpack_require__.y",
      R::THIS_AS_EXPORTS => "top-level-this-exports",
      R::CURRENT_REMOTE_GET_SCOPE => "__webpack_require__.R",
      R::SHARE_SCOPE_MAP => "__webpack_require__.S",
      R::INITIALIZE_SHARING => "__webpack_require__.I",
      R::SCRIPT_NONCE => "__webpack_require__.nc",
      R::RELATIVE_URL => "__webpack_require__.U",
      R::CHUNK_NAME => "__webpack_require__.cn",
      R::RUNTIME_ID => "__webpack_require__.j",
      R::PREFETCH_CHUNK => "__webpack_require__.E",
      R::PREFETCH_CHUNK_HANDLERS => "__webpack_require__.F",
      R::PRELOAD_CHUNK => "__webpack_require__.G",
      R::PRELOAD_CHUNK_HANDLERS => "__webpack_require__.H",
      // rspack only
      R::RSPACK_VERSION => "__webpack_require__.rv",
      R::RSPACK_UNIQUE_ID => "__webpack_require__.ruid",
      R::HAS_CSS_MODULES => "has css modules",
      _ => unreachable!(),
    }
  }
}

impl From<RuntimeGlobals> for Atom {
  fn from(value: RuntimeGlobals) -> Self {
    value.name().into()
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

  #[test]
  fn test_pretty_print() {
    let flags = RuntimeGlobals::PUBLIC_PATH;
    assert_eq!(format!("{flags}"), "__webpack_require__.p");
    let flags = RuntimeGlobals::GET_CHUNK_CSS_FILENAME;
    assert_eq!(format!("{flags}"), "__webpack_require__.k");
  }

  #[test]
  #[should_panic]
  fn test_panic_when_print_multiple_flags() {
    let flags = RuntimeGlobals::PUBLIC_PATH | RuntimeGlobals::GET_CHUNK_CSS_FILENAME;
    print!("{flags}");
  }
}
