import { defineConfig } from '@rspack/cli';
import type { Compiler } from '@rspack/core';
import path from 'node:path';

class Plugin {
  apply(compiler: Compiler) {
    compiler.hooks.compilation.tap('PLUGIN', (compilation) => {
      compilation.hooks.finishModules.tapAsync(
        'PLUGIN',
        (_modules, callback) => {
          const normalResolver = compiler.resolverFactory.get('normal');
          // With query
          normalResolver.resolve(
            {},
            import.meta.dirname,
            './index.js?query',
            {},
            (error, res, req) => {
              expect(
                normalResolver.resolveSync(
                  {},
                  import.meta.dirname,
                  './index.js?query',
                ),
              ).toBe(res);

              expect(error).toBeNull();
              expect(res).toBe(
                path.join(import.meta.dirname, '/index.js?query'),
              );
              // webpack does not have resource field
              expect(req).not.toHaveProperty('resource');
              expect(req?.path).toBe(
                path.join(import.meta.dirname, '/index.js'),
              );
              expect(req?.query).toBe('?query');
              callback();
            },
          );
        },
      );
    });
  }
}

export default defineConfig({
  entry: './index.js',
  plugins: [new Plugin()],
});
