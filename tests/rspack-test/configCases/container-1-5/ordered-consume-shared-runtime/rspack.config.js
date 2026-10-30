const { ModuleFederationPlugin } = require('@rspack/core').container;
const { ConsumeSharedPlugin, ProvideSharedPlugin } =
  require('@rspack/core').sharing;

// Only `ordered` has an ordered consume, added by a separate ConsumeSharedPlugin.
// Both entries must await initialization when they share a runtime. With separate
// runtimes, the scalar entry stays synchronous.
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
