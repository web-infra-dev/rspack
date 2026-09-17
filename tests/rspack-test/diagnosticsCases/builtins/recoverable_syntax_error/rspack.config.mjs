/** @type {import("@rspack/core").Configuration} */
export default {
  entry: './index.tsx',
  module: {
    rules: [
      {
        test: /\.tsx$/,
        use: {
          loader: 'builtin:swc-loader',
        },
      },
    ],
  },
};
