const path = require('path');
const rspack = require('@rspack/core');

const outputPath = path.resolve(
  __dirname,
  '../../../js/config/dll-plugin/concatenated-lib-ident-context',
);

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'production',
  context: __dirname,
  entry: './index.mjs',
  optimization: {
    concatenateModules: true,
    minimize: false,
  },
  output: {
    path: outputPath,
    filename: 'bundle.js',
    library: {
      type: 'commonjs2',
    },
  },
  plugins: [
    new rspack.DllPlugin({
      context: path.resolve(__dirname, '../../..'),
      name: 'dll',
      path: path.resolve(outputPath, 'manifest.json'),
    }),
  ],
};
