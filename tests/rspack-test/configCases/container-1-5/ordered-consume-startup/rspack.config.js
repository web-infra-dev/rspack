const { ModuleFederationPlugin } = require('@rspack/core').container;
const { ConsumeSharedPlugin } = require('@rspack/core').sharing;

// Async startup must be derived from the actual initial consumes of the
// runtime, not only from the high-level `shared` option: these consumes are
// contributed by a separately installed enhanced ConsumeSharedPlugin.
const consume = (name, shareScope, eager) => ({
  [name]: { shareScope, eager, requiredVersion: false },
});

/** @type {import("@rspack/core").Configuration[]} */
module.exports = [
  // ordered initial (eager) consume: startup awaits scope initialization
  {
    entry: './initial.js',
    output: { uniqueName: 'ordered-consume-startup-initial' },
    plugins: [
      new ModuleFederationPlugin({ name: 'ordered_consume_startup_initial' }),
      new ConsumeSharedPlugin({
        enhanced: true,
        consumes: consume('lib', ['primary', 'default'], true),
      }),
    ],
  },
  // ordered consume only behind an async chunk: startup stays synchronous
  {
    entry: './async-only.js',
    output: { uniqueName: 'ordered-consume-startup-async-only' },
    plugins: [
      new ModuleFederationPlugin({
        name: 'ordered_consume_startup_async_only',
      }),
      new ConsumeSharedPlugin({
        enhanced: true,
        consumes: consume('lib', ['primary', 'default'], false),
      }),
    ],
  },
  // mixed scalar + ordered initial consumes
  {
    entry: './mixed.js',
    output: { uniqueName: 'ordered-consume-startup-mixed' },
    plugins: [
      new ModuleFederationPlugin({ name: 'ordered_consume_startup_mixed' }),
      new ConsumeSharedPlugin({
        enhanced: true,
        consumes: {
          ...consume('scalar-lib', 'default', true),
          ...consume('lib', ['primary', 'default'], true),
        },
      }),
    ],
  },
];
