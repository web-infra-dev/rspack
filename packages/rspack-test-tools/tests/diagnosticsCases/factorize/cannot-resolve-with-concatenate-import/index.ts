import { foo } from './reexport';

// should not panic at crates/rspack_plugin_javascript/src/dependency/esm/harmony_import_dependency.rs
foo;