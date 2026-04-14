const {
  experiments: { RslibPlugin },
} = require('@rspack/core');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  externals: {},
  plugins: [
    new RslibPlugin({
      autoCjsNodeBuiltin: true,
    }),
  ],
};
