const { rspack } = require('@rspack/core');

module.exports = ['jsonp', 'import', 'require', 'async-node'].flatMap(
  (loading) =>
    [true, 'relative'].flatMap((url, mode) =>
      ['disabled', 'default-vendors', 'enforced'].map((split) => ({
        name: `${loading}-${mode}-${split}`,
        mode: 'production',
        devtool: false,
        target: ['require', 'async-node'].includes(loading) ? 'node' : 'web',
        experiments: { outputModule: loading === 'import' },
        output: {
          filename: `main-${loading}-${mode}-${split}.${loading === 'import' ? 'mjs' : 'js'}`,
          chunkFilename: `chunk-${loading}-${mode}-${split}-[id].${loading === 'import' ? 'mjs' : 'js'}`,
          publicPath:
            loading === 'jsonp' && split === 'default-vendors'
              ? 'auto'
              : 'https://test.cases/path/',
          chunkLoading: loading === 'async-node' ? loading : undefined,
          module: loading === 'import',
          uniqueName: `url-entry-${loading}-${mode}-${split}`,
        },
        module: {
          parser: { javascript: { url } },
          rules: [
            { dependency: 'url', test: /\.js$/, type: 'javascript/auto' },
          ],
        },
        optimization: {
          minimize: false,
          concatenateModules: split === 'enforced',
          splitChunks:
            split === 'disabled'
              ? false
              : split === 'default-vendors'
                ? { minSize: 0 }
                : {
                    cacheGroups: {
                      target: {
                        test: /url-entry-target/,
                        chunks: 'all',
                        enforce: true,
                        name: 'extracted-target',
                      },
                    },
                  },
        },
        plugins: [
          new rspack.DefinePlugin({
            URL_ENTRY_LOADING: JSON.stringify(loading),
          }),
          (compiler) => {
            compiler.hooks.compilation.tap(
              'CheckExtractedUrlTarget',
              (compilation) => {
                compilation.hooks.processAssets.tap(
                  {
                    name: 'CheckExtractedUrlTarget',
                    stage: rspack.Compilation.PROCESS_ASSETS_STAGE_REPORT,
                  },
                  () => {
                    const target = [...compilation.modules].find((m) =>
                      m
                        .nameForCondition()
                        ?.endsWith('url-entry-target/index.js'),
                    );
                    expect(target).toBeDefined();
                    const targetChunks = [
                      ...compilation.chunkGraph.getModuleChunksIterable(target),
                    ];
                    expect(targetChunks).toHaveLength(1);
                    expect(targetChunks[0].hasRuntime()).toBe(
                      split === 'disabled',
                    );
                    expect(
                      [...compilation.chunks].filter((c) => c.hasRuntime()),
                    ).toHaveLength(2);
                  },
                );
              },
            );
          },
        ],
      })),
    ),
);
