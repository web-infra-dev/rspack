/** @type {import("@rspack/core").Configuration} */
module.exports = {
  externals: {
    './imported_js.bundle0.js': 'commonjs ./imported_js.bundle0.js',
    './reexported_js.bundle0.js': 'commonjs ./reexported_js.bundle0.js',
    './style_module_css.bundle0.js': 'commonjs ./style_module_css.bundle0.js',
  },
  target: 'web',
  mode: 'development',
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
  },
};
