const { rspack } = require('@rspack/core');

module.exports = {
  target: 'node',
  plugins: [
    new rspack.DefinePlugin({
      REPLACEMENT:
        '(() => { function require() { return "replacement"; } return require("./missing-replacement"); })()',
    }),
  ],
};
