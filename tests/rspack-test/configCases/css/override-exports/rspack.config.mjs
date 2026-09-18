/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'development',
  target: 'web',
  module: {
    generator: {
      'css/auto': {
        exportsOnly: false,
      },
    },
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
  },
};
