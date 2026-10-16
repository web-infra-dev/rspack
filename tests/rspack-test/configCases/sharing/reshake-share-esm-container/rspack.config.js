const { ProvideSharedPlugin, TreeShakingSharedPlugin } =
  require('@rspack/core').sharing;

// Regression test for the "× Library name must be unset" bug: a shared
// tree-shaking fallback container with `library: { type: 'module' }` used to
// fail to compile because `SharedContainerPlugin` always set a library `name`,
// which rspack rejects for module-type libraries. With the fix the ESM
// container builds and is emitted as a `.mjs` module.
const shared = {
  'ui-lib': {
    version: '1.0.0',
    treeShaking: {
      mode: 'runtime-infer',
      usedExports: ['Badge', 'MessagePro'],
      filename: '1.0.0/share-entry.mjs',
    },
    requiredVersion: '^1.0.0',
  },
  'ui-lib-dep': {
    version: '1.0.0',
    treeShaking: {
      mode: 'runtime-infer',
      usedExports: ['Message'],
      filename: '1.0.0/share-entry.mjs',
    },
    requiredVersion: '^1.0.0',
  },
};

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  optimization: {
    minimize: true,
    chunkIds: 'named',
    moduleIds: 'named',
  },
  output: {
    chunkFilename: '[id].js',
  },
  // Inherited by the fallback child compiler; required for a module-type
  // container to build.
  experiments: {
    outputModule: true,
  },
  plugins: [
    new ProvideSharedPlugin({
      provides: {
        'ui-lib': {
          shareKey: 'ui-lib',
          version: '1.0.0',
          requiredVersion: '^1.0.0',
          treeShakingMode: 'runtime-infer',
        },
        'ui-lib-dep': {
          shareKey: 'ui-lib-dep',
          version: '1.0.0',
          requiredVersion: '^1.0.0',
          treeShakingMode: 'runtime-infer',
        },
      },
      enhanced: true,
    }),
    new TreeShakingSharedPlugin({
      secondary: true,
      mfConfig: {
        name: 'esm_container_tree_shaking_share',
        library: {
          type: 'module',
        },
        shared,
        treeShakingSharedExcludePlugins: ['ProvideSharedPlugin'],
      },
    }),
  ],
};
