const { container } = require('@rspack/core');

const { ModuleFederationPlugin } = container;

class RspackModuleFederationPlugin {
  name = 'RspackModuleFederationPlugin';

  apply(compiler) {
    const outputPath = compiler.options.output?.path || '';
    if (outputPath.includes('independent-packages')) {
      throw new Error(
        'RspackModuleFederationPlugin should not be applied to shared fallback compilers',
      );
    }
  }
}

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  // entry: './index.js',
  target: 'async-node',
  optimization: {
    minimize: true,
    chunkIds: 'named',
    moduleIds: 'named',
  },
  output: {
    chunkFilename: '[id].js',
  },
  plugins: [
    new RspackModuleFederationPlugin(),
    new ModuleFederationPlugin({
      name: 'tree_shaking_share',
      manifest: true,
      runtimePlugins: [require.resolve('./runtime-plugin.js')],
      library: {
        type: 'commonjs-module',
        name: 'tree_shaking_share',
      },
      shared: {
        'ui-lib': {
          requiredVersion: '*',
          treeShaking: {
            mode: 'runtime-infer',
          },
        },
        'ui-lib-es': {
          requiredVersion: '*',
          treeShaking: {
            mode: 'runtime-infer',
          },
        },
        'ui-lib-dynamic-specific-export': {
          requiredVersion: '*',
          treeShaking: {
            mode: 'runtime-infer',
          },
        },
        'ui-lib-dynamic-default-export': {
          requiredVersion: '*',
          treeShaking: {
            mode: 'runtime-infer',
          },
        },
        'ui-lib-side-effect': {
          requiredVersion: '*',
          treeShaking: {
            mode: 'runtime-infer',
          },
        },
      },
    }),
  ],
};
