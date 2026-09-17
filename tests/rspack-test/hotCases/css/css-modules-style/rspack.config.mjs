/** @type {import("@rspack/core").Configuration} */
export default {
  entry: {
    main: './index.js',
  },
  module: {
    parser: {
      'css/module': {
        namedExports: false,
      },
    },
    rules: [
      {
        test: /\.module\.css$/,
        type: 'css/module',
        generator: {
          exportsOnly: true,
        },
      },
    ],
  },
};
