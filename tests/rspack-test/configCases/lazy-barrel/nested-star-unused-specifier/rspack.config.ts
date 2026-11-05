import path from 'node:path';
import { defineConfig, definePlugin } from '@rspack/cli';

const lazyModules = [
  'ui/theme/fonts.js',
  'ui/theme/spacing.js',
  'ui/theme/default-theme.js',
  'kit/theme/more.js',
  'lib/foo.js',
  'lib/other.js',
].map((file) => path.resolve(import.meta.dirname, file));

export default defineConfig({
  mode: 'production',
  module: { parser: { javascript: { exportsPresence: 'error' } } },
  optimization: {
    sideEffects: true,
    providedExports: true,
    usedExports: true,
    minimize: false,
    concatenateModules: false,
  },
  plugins: [
    definePlugin((compiler) => {
      const createdModules = new Set<string>();
      compiler.hooks.thisCompilation.tap(
        'Test',
        (_compilation, { normalModuleFactory }) => {
          normalModuleFactory.hooks.createModule.tap('Test', (data) => {
            createdModules.add(data.resourceResolveData.resource);
          });
        },
      );
      compiler.hooks.done.tap('Test', () => {
        for (const module of lazyModules) {
          expect(createdModules.has(module)).toBe(false);
        }
      });
    }),
  ],
});
