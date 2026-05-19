module.exports = {
  optimization: {
    runtimeChunk: false,
  },
  plugins: [
    function emptyEntryAssetPlugin(compiler) {
      compiler.hooks.compilation.tap(
        'empty-entry-asset-plugin',
        (compilation) => {
          compilation.hooks.processAssets.tap(
            {
              name: 'empty-entry-asset-plugin',
              stage:
                compiler.rspack.Compilation.PROCESS_ASSETS_STAGE_OPTIMIZE_HASH,
            },
            (assets) => {
              if (!assets['main.mjs']) {
                return;
              }

              compilation.updateAsset(
                'main.mjs',
                new compiler.rspack.sources.RawSource(''),
              );
            },
          );
        },
      );
    },
  ],
};
