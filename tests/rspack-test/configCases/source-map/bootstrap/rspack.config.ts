import { defineConfig } from '@rspack/cli';
import { rspack, type RspackOptions } from '@rspack/core';

const configs: RspackOptions[] = [];

for (const minimize of [false, true]) {
  for (const splitRuntime of [false, true]) {
    for (const usePlugin of [false, true]) {
      const index = configs.length;
      const runtimeFilename = splitRuntime
        ? `runtime${index}.js`
        : `bundle${index}.js`;
      configs.push({
        entry: { [`bundle${index}`]: './index.js' },
        devtool: usePlugin ? false : 'source-map',
        output: { filename: '[name].js' },
        node: { __dirname: false, __filename: false },
        optimization: {
          minimize,
          concatenateModules: false,
          runtimeChunk: splitRuntime ? { name: `runtime${index}` } : false,
        },
        externals: ['source-map'],
        externalsType: 'commonjs',
        plugins: [
          ...(usePlugin
            ? [new rspack.SourceMapDevToolPlugin({ filename: '[file].map' })]
            : []),
          new rspack.DefinePlugin({
            RUNTIME_FILENAME: JSON.stringify(runtimeFilename),
            EXPECT_STARTUP: JSON.stringify(!splitRuntime),
          }),
        ],
      });
    }
  }
}

export default defineConfig(configs);
