import { defineConfig, definePlugin } from '@rspack/cli';

export default defineConfig({
  entry: {
    bundle0: './index',
    b: './b',
  },
  output: {
    filename: '[name].js',
  },
  plugins: [
    definePlugin(function () {
      this.hooks.compilation.tap('TestPlugin', function (compilation) {
        compilation.hooks.processAssets.tap('TestPlugin', function () {
          delete compilation.assets['b.js'];
        });
      });
    }),
  ],
});
