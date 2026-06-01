const { ProvideSharedPlugin, TreeShakingSharedPlugin } =
  require('@rspack/core').sharing;
const path = require('path');

const shared = {
  'ui-lib': {
    version: '1.0.0',
    treeShaking: {
      mode: 'runtime-infer',
      usedExports: ['Badge', 'MessagePro'],
    },
    requiredVersion: '^1.0.0',
  },
  'ui-lib-dep': {
    version: '1.0.0',
    treeShaking: {
      mode: 'runtime-infer',
      usedExports: ['Message'],
    },
    requiredVersion: '^1.0.0',
  },
};

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  // entry:'./index.js',
  optimization: {
    minimize: true,
    chunkIds: 'named',
    moduleIds: 'named',
  },
  output: {
    chunkFilename: '[id].js',
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
        name: 'secondary_tree_shaking_share',
        library: {
          type: 'commonjs2',
        },
        shared,
        treeShakingSharedExcludePlugins: ['ProvideSharedPlugin'],
        treeShakingSharedPlugins: [
          path.resolve(__dirname, './CustomPlugin.js'),
        ],
      },
    }),
  ],
};
