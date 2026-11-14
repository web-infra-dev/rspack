const assert = require('node:assert/strict');
const rspack = require('@rspack/core');

module.exports = ['independent', 'shared', 'depend-on'].flatMap((layout) =>
  [undefined, false, true, /^filtered$/g, /^(filtered|untouched)$/g].map(
    (removeUnusedRuntime, optionIndex) => {
      const name = `${layout}-${optionIndex}`;
      const sharedRuntime = layout !== 'independent';
      const filtered = optionIndex < 2 || (sharedRuntime && optionIndex === 3);
      const untouched = optionIndex < 2 || optionIndex === 3;
      return {
        name,
        mode: 'development',
        target: 'async-node',
        devtool: false,
        entry: {
          filtered: './filtered.js',
          untouched:
            layout === 'depend-on'
              ? { import: './untouched.js', dependOn: 'filtered' }
              : './untouched.js',
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
            EXPECT_FILTERED_FEDERATION: JSON.stringify(filtered),
            EXPECT_UNTOUCHED_FEDERATION: JSON.stringify(untouched),
          }),
          new rspack.container.ModuleFederationPlugin({
            name: `provider_${name.replaceAll('-', '_')}`,
            removeUnusedRuntime,
            runtimePlugins: [require.resolve('./runtime-plugin.js')],
            remotes: {
              unused:
                "promise Promise.resolve({ get: () => Promise.resolve(() => 'unused'), init: () => {} })",
            },
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
                'CheckProviderExclusion',
                (compilation) => {
                  compilation.hooks.processAssets.tap(
                    {
                      name: 'CheckProviderExclusion',
                      stage: rspack.Compilation.PROCESS_ASSETS_STAGE_REPORT,
                    },
                    () => {
                      if (!filtered && !untouched) {
                        for (const { source } of compilation.getAssets()) {
                          assert.doesNotMatch(
                            String(source.source()),
                            /provider-only-marker|\.federation\.instance\s*=/,
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
      };
    },
  ),
);
