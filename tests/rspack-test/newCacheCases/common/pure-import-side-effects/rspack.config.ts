import { defineConfig, definePlugin } from '@rspack/cli';

let buildIndex = 0;
export default defineConfig({
  mode: 'production',
  output: {
    module: true,
    library: { type: 'module' },
    filename: 'bundle.mjs',
  },
  optimization: {
    concatenateModules: false,
    innerGraph: false,
    inlineExports: false,
    usedExports: false,
    minimize: true,
    sideEffects: true,
  },
  cache: { type: 'persistent' },
  plugins: [
    definePlugin({
      apply(compiler) {
        compiler.hooks.compilation.tap('PureImportCache', (compilation) => {
          compilation.hooks.processAssets.tap(
            {
              name: 'PureImportCache',
              stage:
                compiler.rspack.Compilation.PROCESS_ASSETS_STAGE_OPTIMIZE_SIZE -
                1,
            },
            () => {
              const source = String(
                compilation.getAsset('bundle.mjs')!.source.source(),
              );
              for (const request of ['direct', 'transitive']) {
                const annotated = `/*#__PURE__*/ __webpack_require__("./${request}.js")`;
                expect(source.includes(annotated)).toBe(buildIndex !== 2);
              }
            },
          );
        });
        compiler.hooks.done.tap('PureImportCache', () => {
          buildIndex++;
        });
      },
    }),
  ],
});
