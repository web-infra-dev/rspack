const { ModuleFederationPlugin } = require('@rspack/core').container;
const { ConsumeSharedPlugin, ProvideSharedPlugin } =
  require('@rspack/core').sharing;

// Two entries share one runtime chunk. Only `ordered` has an ordered (array)
// scope consume (contributed by a separately installed enhanced
// ConsumeSharedPlugin, invisible to the high-level `shared` option), but the
// runtime installs the initial consumes of both entries behind that scope's
// initialization, so the startup plan must be decided per runtime and applied
// to every entry it serves. The third config keeps separate runtimes as a
// control: the scalar entry stays synchronous there.
const base = (dir, runtimeChunk) => ({
  entry: {
    ordered: './ordered.js',
    scalar: './scalar.js',
  },
  output: {
    filename: `${dir}/[name].js`,
    chunkFilename: `${dir}/[name].js`,
    uniqueName: `ordered-consume-shared-runtime-${dir}`,
  },
  optimization: { runtimeChunk },
  plugins: [
    new ModuleFederationPlugin({
      name: `ordered_consume_shared_runtime_${dir.replace(/-/g, '_')}`,
    }),
    new ProvideSharedPlugin({
      enhanced: true,
      provides: { 'scalar-lib': { shareKey: 'scalar-lib', eager: true } },
    }),
    new ConsumeSharedPlugin({
      enhanced: true,
      consumes: {
        lib: {
          shareScope: ['primary', 'default'],
          eager: true,
          requiredVersion: false,
        },
        'scalar-lib': { eager: true, requiredVersion: false },
      },
    }),
  ],
});

/** @type {import("@rspack/core").Configuration[]} */
module.exports = [
  base('shared-scalar-first', 'single'),
  base('shared-ordered-first', 'single'),
  base('separate', false),
];
