const { ProvideSharedPlugin } = require('@rspack/core').sharing;
const { ModuleFederationPlugin } = require('@rspack/core').container;

// `./unused-provider` is not imported by the entry, so it is only built by the
// `add_include` call in `finish_make`. Building it imports `package`, which is
// a match-provided share first discovered during that same `add_include`.
// This used to deadlock: `finish_make` held the provider map read lock while
// the discovered provider waited for the write lock.
const shared = {
  './unused-provider': {
    shareKey: 'unused-provider',
    version: '1.0.0',
  },
  package: { requiredVersion: false },
};

/** @type {import("@rspack/core").Configuration[]} */
module.exports = [
  // v1 (standalone provider)
  {
    output: { uniqueName: 'provide-discovered-during-add-include-v1' },
    plugins: [
      new ProvideSharedPlugin({
        provides: {
          './unused-provider': shared['./unused-provider'],
          package: { shareKey: 'package' },
        },
      }),
    ],
  },
  // enhanced (v1.5 runtime)
  {
    output: { uniqueName: 'provide-discovered-during-add-include-enhanced' },
    plugins: [
      new ModuleFederationPlugin({
        name: 'provide_discovered_during_add_include',
        shared,
      }),
    ],
  },
];
