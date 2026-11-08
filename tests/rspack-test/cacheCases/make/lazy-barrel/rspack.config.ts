import path from 'node:path';
import { defineConfig, definePlugin } from '@rspack/cli';

export default defineConfig({
  context: import.meta.dirname,
  optimization: {
    // avoid analyze side effects that will change index.js dependencies at HMR
    sideEffects: 'flag',
  },
  cache: {
    type: 'persistent',
  },
  plugins: [
    definePlugin((compiler) => {
      const createdModules = new Set<string>();
      compiler.hooks.compilation.tap(
        'test',
        (_compilation, { normalModuleFactory }) => {
          normalModuleFactory.hooks.createModule.tap('test', (data) => {
            createdModules.add(data.resourceResolveData.resource);
          });
        },
      );
      compiler.hooks.done.tap('Test', () => {
        expect(
          createdModules.has(path.resolve(import.meta.dirname, 'lib/c.js')),
        ).toBe(false);
      });
    }),
  ],
});
