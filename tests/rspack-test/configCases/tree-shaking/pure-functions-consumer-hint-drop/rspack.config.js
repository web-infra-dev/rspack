/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'production',
  target: 'node',
  optimization: {
    sideEffects: true,
    innerGraph: true,
    usedExports: true,
    minimize: false,
    concatenateModules: false,
  },
  experiments: {
    pureFunctions: true,
  },
  module: {
    rules: [
      {
        // Hint is on the CONSUMER (producer.js, where the call is parsed), so
        // `identity` resolves via the trust-at-call-site Direct path.
        test: /producer\.js$/,
        parser: {
          pureFunctions: ['identity'],
        },
      },
    ],
  },
};
