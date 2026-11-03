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
  },
  output: { filename: 'main.js' },
  plugins: [
    definePlugin({
      apply(compiler) {
        compiler.hooks.done.tap('AdvancedTreeShaking', (stats) => {
          const output = String(
            stats.compilation.getAsset('main.js')!.source.source(),
          );
          if (buildIndex === 0) {
            expect(output).toContain('./bridge.js');
            expect(output).toContain('./dep.js');
            expect(output).toContain('./tracker.js');
            expect(output).toContain('impure');
          } else {
            expect(output).not.toContain('./bridge.js');
            expect(output).not.toContain('./dep.js');
            expect(output).not.toContain('./tracker.js');
            expect(output).not.toContain('impure');
          }
          buildIndex++;
        });
      },
    }),
  ],
});
