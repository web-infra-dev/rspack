module.exports = {
  externals: {
    './continue-on-policy-creation-failure.web.js':
      'commonjs ./continue-on-policy-creation-failure.web.js',
  },
  target: 'web',
  output: {
    chunkFilename: '[name].web.js',
    crossOriginLoading: 'anonymous',
    trustedTypes: {
      policyName: 'CustomPolicyName',
      onPolicyCreationFailure: 'continue',
    },
  },
  performance: {
    hints: false,
  },
  optimization: {
    minimize: false,
  },
};
