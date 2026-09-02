const BeforeLoadersPlugin = require('./plugin.js');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  module: {
    rules: [
      {
        test: /(mutate|remove|untouched)\.js$/,
        use: [
          {
            loader: require.resolve('./tag-loader.js'),
            options: { tag: '+config' },
          },
        ],
      },
    ],
  },
  plugins: [new BeforeLoadersPlugin()],
};
