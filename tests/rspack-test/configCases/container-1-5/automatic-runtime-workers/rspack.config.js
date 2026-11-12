const rspack = require('@rspack/core');

module.exports = [
  undefined,
  false,
  true,
  /^(plain-worker|remote-worker|shared-worker|comment-worker)$/gi,
].map((excludeBundlerRuntime, index) => ({
  name: `worker-option-${index}`,
  target: 'async-node',
  mode: 'development',
  devtool: false,
  optimization: { minimize: false },
  output: {
    filename: `worker-option-${index}/[name].js`,
    chunkFilename: `worker-option-${index}/[id].js`,
    uniqueName: `worker-option-${index}`,
  },
  plugins: [
    new rspack.DefinePlugin({
      EXPECT_PARENT_FEDERATION: JSON.stringify(index !== 2),
      EXPECT_NAMED_FEDERATION: JSON.stringify(index < 2),
      EXPECT_UNNAMED_FEDERATION: JSON.stringify(index !== 2),
    }),
    new rspack.container.ModuleFederationPlugin({
      name: 'worker_host',
      excludeBundlerRuntime,
      shareStrategy: 'loaded-first',
      remotes: {
        remote:
          "promise Promise.resolve({ get: () => Promise.resolve(() => 'remote value'), init: () => {} })",
      },
      shared: {
        'shared-lib': {
          import: './shared.js',
          version: '1.0.0',
          requiredVersion: false,
        },
      },
    }),
  ],
}));
