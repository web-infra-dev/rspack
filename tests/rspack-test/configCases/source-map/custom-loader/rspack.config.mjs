/** @type {import("@rspack/core").Configuration} */
export default {
  devtool: false,
  plugins: [
    (compiler) => {
      new compiler.rspack.SourceMapDevToolPlugin({}).apply(compiler);
    },
  ],
  module: {
    rules: [
      {
        loader: './loader.js',
      },
    ],
  },
};
