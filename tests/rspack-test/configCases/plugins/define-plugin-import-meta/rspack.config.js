var DefinePlugin = require('@rspack/core').DefinePlugin;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  plugins: [
    new DefinePlugin({
      'import.meta.env.MODE': '"production"',
      'import.meta.MY_ENV': JSON.stringify('canary'),
      FOO: 'import.meta.unknownProperty',
    }),
  ],
};
