/** @type {import("@rspack/core").Configuration} */
export default {
  entry: {
    bundle0: './index',
    b: './b',
  },
  output: {
    filename: '[name].js',
  },
  plugins: [
    function () {
      this.hooks.compilation.tap('TestPlugin', function (compilation) {
        compilation.hooks.processAssets.tap('TestPlugin', function (assets) {
          delete compilation.assets['b.js'];
        });
      });
    },
  ],
};
