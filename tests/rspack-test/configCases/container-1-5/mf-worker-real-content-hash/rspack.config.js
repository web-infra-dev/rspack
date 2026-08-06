const assert = require('node:assert');
const rspack = require('@rspack/core');
const { ModuleFederationPlugin } = rspack.container;

const sharedKey = 'mf-worker-v2-provider-marker';
const providerPattern = new RegExp(
  `initializeSharingData\\s*=\\s*\\{\\s*scopeToSharingDataMapping\\s*:\\s*\\{\\s*(?:"default"|default)\\s*:\\s*\\[\\s*\\{\\s*name\\s*:\\s*"${sharedKey}"`,
);
const federationRuntimePattern = /\.federation\.instance\s*=/;

class CheckWorkerSharingPlugin {
  apply(compiler) {
    compiler.hooks.thisCompilation.tap(
      'CheckWorkerSharingPlugin',
      (compilation) => {
        compilation.hooks.processAssets.tap(
          {
            name: 'CheckWorkerSharingPlugin',
            stage: rspack.Compilation.PROCESS_ASSETS_STAGE_REPORT,
          },
          (assets) => {
            const sources = Object.values(assets).map((asset) =>
              String(asset.source()),
            );
            const workers = sources.filter(
              (source) =>
                source.includes('worker a') || source.includes('worker b'),
            );
            assert.strictEqual(
              workers.length,
              2,
              'both workers should be emitted',
            );

            for (const worker of workers) {
              assert.doesNotMatch(
                worker,
                providerPattern,
                'worker entrypoints should not contain the shared provider',
              );
              assert.match(
                worker,
                federationRuntimePattern,
                'worker entrypoints should retain the federation runtime',
              );
            }

            assert(
              sources.some((source) => providerPattern.test(source)),
              'initial entrypoints should retain the shared provider',
            );
          },
        );
      },
    );
  }
}

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'production',
  output: {
    filename: '[name].js',
    chunkFilename: '[id].[contenthash].js',
  },
  optimization: {
    realContentHash: true,
  },
  plugins: [
    new ModuleFederationPlugin({
      name: 'host',
      filename: 'remoteEntry.js',
      exposes: {
        './shared': './shared.js',
      },
      shared: {
        [sharedKey]: {
          import: './shared.js',
          singleton: true,
          version: '1.0.0',
        },
      },
    }),
    new CheckWorkerSharingPlugin(),
  ],
};
