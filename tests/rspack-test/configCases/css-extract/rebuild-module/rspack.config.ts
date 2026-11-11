import { defineConfig, definePlugin } from '@rspack/cli';
import { CssExtractRspackPlugin } from '@rspack/core';

export default defineConfig({
  module: {
    rules: [
      {
        test: /\.css$/,
        use: [CssExtractRspackPlugin.loader, 'css-loader', './loader.mjs'],
        type: 'javascript/auto',
      },
    ],
  },
  plugins: [
    new CssExtractRspackPlugin({
      chunkFilename: 'bundle.css',
    }),
    definePlugin((compiler) => {
      let initial = true;
      compiler.hooks.finishMake.tapAsync(
        'TEST_PLUGIN',
        (compilation, callback) => {
          const modules = compilation.modules;
          if (!initial) {
            return callback();
          }
          initial = false;
          const cssModule = Array.from(modules).find(
            (module) =>
              module.identifier().includes('.css') &&
              module.type === 'javascript/auto',
          );
          compilation.rebuildModule(cssModule!, (error) => callback(error));
        },
      );
    }),
  ],
});
