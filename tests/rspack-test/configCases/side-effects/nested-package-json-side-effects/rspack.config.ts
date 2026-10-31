import { defineConfig, definePlugin } from '@rspack/cli';
import path from 'node:path';

const packageJson = (...segments: string[]) =>
  path.join(import.meta.dirname, 'node_modules', ...segments, 'package.json');

export default defineConfig({
  mode: 'production',
  optimization: {
    sideEffects: true,
    usedExports: true,
    providedExports: true,
    concatenateModules: false,
    minimize: false,
  },
  plugins: [
    definePlugin((compiler) => {
      compiler.hooks.afterEmit.tap('Test', (compilation) => {
        // the walk reads ancestors the resolver never looks at, so it reports
        // them itself — otherwise an edited sideEffects would not invalidate
        expect(
          compilation.fileDependencies.has(
            packageJson('unnamed-inner-sef', 'dist'),
          ),
        ).toBe(true);
        expect(
          compilation.missingDependencies.has(
            packageJson('missing-mid-sef', 'dist'),
          ),
        ).toBe(true);
      });
    }),
  ],
});
