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
          normalResolver.resolve(
            {},
            import.meta.dirname,
            './index.js',
            {},
            (error, res, req) => {
              expect(
                normalResolver.resolveSync(
                  {},
                  import.meta.dirname,
                  './index.js',
                ),
              ).toBe(res);

              expect(error).toBeNull();
              expect(res).toBe(path.join(import.meta.dirname, '/index.js'));
              // webpack does not have resource field
              expect(Reflect.get(req!, 'resource')).toBe(undefined);
              expect(req?.path).toBe(
                path.join(import.meta.dirname, '/index.js'),
              );
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
