/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'development',
  target: 'web',
  devtool: 'source-map',
  module: {
    generator: {
      'css/auto': {
        exportsOnly: false,
      },
    },
    rules: [
      {
        test: /\.css$/,
        use: ['./assert-source-map-loader.js', 'builtin:lightningcss-loader'],
        sideEffects: true,
        type: 'css/auto',
      },
    ],
  },
};
