const assert = require('node:assert/strict');
const rspack = require('@rspack/core');

module.exports = ['independent', 'shared', 'depend-on'].flatMap((layout) =>
  [false, true].map((asyncStartup) => {
    const name = `${layout}-${asyncStartup ? 'async' : 'sync'}`;
    const sharedRuntime = layout !== 'independent';
    return {
      name,
      mode: 'development',
      target: 'async-node',
      devtool: false,
      entry: {
        plain: './plain.js',
        remote:
          layout === 'depend-on'
            ? { import: './remote.js', dependOn: 'plain' }
            : './remote.js',
        consume:
          layout === 'depend-on'
            ? { import: './consume.js', dependOn: 'plain' }
            : './consume.js',
      },
      output: {
        filename: `${name}/[name].js`,
        chunkFilename: `${name}/[id].js`,
        uniqueName: name,
      },
      optimization: {
        minimize: false,
        concatenateModules: false,
        runtimeChunk: layout === 'shared' ? 'single' : false,
      },
      plugins: [
        new rspack.DefinePlugin({
          EXPECT_FEDERATION: JSON.stringify(sharedRuntime),
        }),
        new rspack.container.ModuleFederationPlugin({
          name: `container_${name.replaceAll('-', '_')}`,
          filename: `${name}/container.js`,
          library: { type: 'commonjs-module' },
          exposes: { './exposed': './exposed.js' },
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
          experiments: { asyncStartup },
        }),
        new rspack.sharing.ProvideSharedPlugin({
          enhanced: true,
          provides: {
            './provided.js': {
              shareKey: 'provider-only',
              version: '1.0.0',
              eager: true,
            },
          },
        }),
        {
          apply(compiler) {
            compiler.hooks.thisCompilation.tap(
              'CheckAutomaticRuntime',
              (compilation) => {
                compilation.hooks.processAssets.tap(
                  {
                    name: 'CheckAutomaticRuntime',
                    stage: rspack.Compilation.PROCESS_ASSETS_STAGE_REPORT,
                  },
                  () => {
                    if (!sharedRuntime) {
                      const source = String(
                        compilation
                          .getAsset(`${name}/plain.js`)
                          .source.source(),
                      );
                      assert.doesNotMatch(
                        source,
                        /initializeSharingData|prevStartup|provider-only-marker/,
                      );
                    }
                    const sources = compilation
                      .getAssets()
                      .map(({ source }) => String(source.source()));
                    assert(
                      sources.some((source) =>
                        source.includes('provider-only-marker'),
                      ),
                    );
                  },
                );
              },
            );
          },
        },
      ],
    };
  }),
);
