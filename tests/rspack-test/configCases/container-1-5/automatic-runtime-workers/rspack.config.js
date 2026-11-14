const assert = require('node:assert/strict');
const rspack = require('@rspack/core');

const federationRuntimePattern = /\.federation\.instance\s*=/;

module.exports = [
  undefined,
  false,
  true,
  /^(plain-worker|remote-worker|shared-worker|comment-worker|remote-entry-local-worker)$/gi,
].map((removeUnusedRuntime, index) => ({
  name: `worker-option-${index}`,
  target: 'async-node',
  mode: 'development',
  devtool: false,
  entry: {
    main: './index.js',
    'local-a': './local-entry.js',
    'local-b': './local-second-entry.js',
    remote: './remote-entry.js',
    consume: './consume-entry.js',
  },
  optimization: { minimize: false, runtimeChunk: false },
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
      removeUnusedRuntime,
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
    {
      apply(compiler) {
        compiler.hooks.thisCompilation.tap(
          'CheckMultiEntryWorkerRuntime',
          (compilation) => {
            compilation.hooks.processAssets.tap(
              {
                name: 'CheckMultiEntryWorkerRuntime',
                stage: rspack.Compilation.PROCESS_ASSETS_STAGE_REPORT,
              },
              () => {
                for (const entry of [
                  'main',
                  'local-a',
                  'local-b',
                  'remote',
                  'consume',
                ]) {
                  const source = String(
                    compilation
                      .getAsset(`worker-option-${index}/${entry}.js`)
                      .source.source(),
                  );
                  const expected =
                    entry === 'remote' || entry === 'consume' || index !== 2;
                  (expected ? assert.match : assert.doesNotMatch)(
                    source,
                    federationRuntimePattern,
                    `${entry} should only include the federation runtime when required or outside the filtering scope`,
                  );
                }
                const localWorkers = [...compilation.chunks].filter((chunk) =>
                  [
                    'plain-worker',
                    'comment-worker',
                    'remote-entry-local-worker',
                  ].includes(chunk.name),
                );
                assert.strictEqual(localWorkers.length, 3);
                for (const chunk of localWorkers) {
                  for (const file of chunk.files) {
                    if (!file.endsWith('.js')) continue;
                    const source = String(
                      compilation.getAsset(file).source.source(),
                    );
                    (index < 2 ? assert.match : assert.doesNotMatch)(
                      source,
                      federationRuntimePattern,
                      `${chunk.name} should omit unused federation runtime when filtering is enabled`,
                    );
                  }
                }
              },
            );
          },
        );
      },
    },
  ],
}));
