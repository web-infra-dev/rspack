/** @type {import("@rspack/core").Configuration} */
export default {
  module: {
    generator: {
      'css/auto': {
        localIdentName: '[path][name]-[local]',
      },
    },
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
  },
};
