import { defineConfig, definePlugin } from '@rspack/cli';

let buildIndex = 0;

export default defineConfig({
  mode: 'production',
  target: 'node',
  entry: './index.js',
  cache: { type: 'persistent' },
  experiments: { pureFunctions: true },
  optimization: {
    sideEffects: true,
    innerGraph: true,
    usedExports: true,
    concatenateModules: false,
    inlineExports: false,
  },
  output: { filename: 'main.js' },
  plugins: [
    definePlugin({
      apply(compiler) {
        compiler.hooks.compilation.tap(
          'InnerGraphDeferredPure',
          (compilation) => {
            compilation.hooks.processAssets.tap(
              {
                name: 'InnerGraphDeferredPure',
                stage:
                  compiler.rspack.Compilation
                    .PROCESS_ASSETS_STAGE_OPTIMIZE_SIZE - 1,
              },
              () => {
                const output = String(
                  compilation.getAsset('main.js')!.source.source(),
                );
                if (buildIndex === 0) {
                  expect(output).toContain('./dep.js');
                  expect(output).not.toContain(
                    'unused pure expression or super',
                  );
                } else {
                  expect(output).toContain('unused pure expression or super');
                }
              },
            );
          },
        );
        compiler.hooks.done.tap('InnerGraphDeferredPure', (stats) => {
          const output = String(
            stats.compilation.getAsset('main.js')!.source.source(),
          );
          if (buildIndex === 0) {
            expect(output).toContain('inner-graph-impure');
          } else {
            expect(output).not.toContain('inner-graph-impure');
            expect(
              stats.toJson({ all: false, cacheInfo: true }).cacheInfo![
                '|Compilation/modules'
              ].reused,
            ).toBeGreaterThanOrEqual(2);
          }
          buildIndex++;
        });
      },
    }),
  ],
});
