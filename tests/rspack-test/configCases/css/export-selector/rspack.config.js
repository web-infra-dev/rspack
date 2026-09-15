/** @type {import("@rspack/core").Configuration} */
module.exports = {
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
    './imported_js.bundle0.js': 'commonjs ./imported_js.bundle0.js',
    './reexported_js.bundle0.js': 'commonjs ./reexported_js.bundle0.js',
  },
  target: 'web',
  node: {
    __dirname: false,
    __filename: false,
  },
  module: {
    generator: {
      'css/auto': {
        exportsConvention: 'camel-case-only',
        localIdentName: '[local]',
        exportsOnly: false,
      },
    },
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
  },
  mode: 'development',
};
