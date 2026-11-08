import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

/**
 * Create a webpack configuration for a given target.
 * @param target target environment
 * @param concatenateModules whether to concatenate modules
 * @returns webpack configuration
 */
function createConfig(target: 'node' | 'web', concatenateModules: boolean) {
  return defineConfig({
    name: `${target}-${concatenateModules ? 'concat' : 'no-concat'}`,
    devtool: false,
    target,
    mode: 'development',
    experiments: {
      css: true,
    },
    optimization: {
      chunkIds: 'named',
      concatenateModules,
    },
    module: {
      rules: [
        {
          test: /\.css$/,
          type: 'css/module',
          parser: {
            exportType: 'style',
          },
        },
      ],
    },
    plugins: [
      new rspack.DefinePlugin({
        'process.env.BROWSER': JSON.stringify(target === 'web'),
      }),
    ],
  });
}
export default defineConfig([
  createConfig('node', false),
  createConfig('node', true),
  createConfig('web', false),
  createConfig('web', true),
]);
