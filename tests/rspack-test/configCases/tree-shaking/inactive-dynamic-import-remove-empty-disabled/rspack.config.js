/** @type {import("@rspack/core").Configuration} */
module.exports = {
  target: 'node',
  output: {
    chunkFilename: '[name].js',
  },
  optimization: {
    concatenateModules: false,
    innerGraph: true,
    minimize: false,
    providedExports: true,
    removeEmptyChunks: false,
    sideEffects: true,
    usedExports: true,
  },
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.compilation.tap(
          'InactiveDynamicImportRemoveEmptyDisabled',
          (compilation) => {
            compilation.hooks.afterSeal.tap(
              'InactiveDynamicImportRemoveEmptyDisabled',
              () => {
                expect(compilation.namedChunks.get('dead')).toBeTruthy();
                expect(compilation.namedChunkGroups.get('dead')).toBeTruthy();
              },
            );
          },
        );
      },
    },
  ],
};
