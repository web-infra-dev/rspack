const BeforeLoadersPlugin = require('./plugin.js');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  module: {
    rules: [
      {
        test: /(mutate|remove|spread|untouched)\.js$/,
        use: [
          {
            loader: require.resolve('./tag-loader.js'),
            options: { tag: '+config' },
          },
        ],
      },
      {
        test: /typed\.js$/,
        use: [{ loader: require.resolve('./noop-loader.cjs') }],
      },
    ],
  },
  plugins: [new BeforeLoadersPlugin()],
};
