const { rspack } = require('@rspack/core');

module.exports = {
  target: 'node',
  module: {
    parser: {
      javascript: {
        requireAlias: true,
        createRequire: true,
      },
    },
  },
  plugins: [
    new rspack.DefinePlugin({
      CALLER_EXPRESSION: 'require("./value")',
      LOCAL_EXPRESSION:
        '(() => { const require = () => "fragment"; return require("./missing-fragment"); })()',
    }),
  ],
};
