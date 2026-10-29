import { defineConfig, definePlugin } from '@rspack/cli';

let buildIndex = 0;
let firstOutput = '';

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
        compiler.hooks.done.tap('InnerGraphDeferredPure', (stats) => {
          const output = String(
            stats.compilation.getAsset('main.js')!.source.source(),
          );
          if (buildIndex === 0) {
            expect(output).toContain('inner-graph-impure');
            firstOutput = output;
          } else {
            expect(output).not.toContain('inner-graph-impure');
            expect(output).toContain('"pure"');
            expect(output).not.toBe(firstOutput);
          }
          buildIndex++;
        });
      },
    }),
  ],
});
