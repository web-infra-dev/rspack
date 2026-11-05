import path from 'node:path';
import { defineConfig, definePlugin } from '@rspack/cli';

let buildIndex = 0;
const valueFile = path.join(import.meta.dirname, 'value.js');

export default defineConfig({
  devtool: false,
  entry: './index.js',
  cache: { type: 'persistent' },
  output: { filename: 'main.js' },
  plugins: [
    definePlugin({
      apply(compiler) {
        compiler.hooks.done.tap('CacheRestoreChangeDetection', (stats) => {
          const output = String(
            stats.compilation.getAsset('main.js')!.source.source(),
          );
          if (buildIndex === 0) {
            expect(output).toContain('initial value');
          } else {
            expect(output).toContain('updated value');
            expect(
              [...stats.compilation.fileDependencies].some(
                (dependency) => path.basename(dependency) === 'value.js',
              ),
            ).toBe(true);
            expect(
              [...stats.compilation.fileDependencies].some(
                (dependency) => dependency === valueFile,
              ),
            ).toBe(true);
          }
          buildIndex++;
        });
      },
    }),
  ],
});
