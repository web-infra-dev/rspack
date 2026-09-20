import path from 'node:path';

class Plugin {
  /**
   * @param {import("@rspack/core").Compiler} compiler
   */
  apply(compiler) {
    compiler.hooks.compilation.tap('PLUGIN', (compilation) => {
      compilation.hooks.finishModules.tapAsync(
        'PLUGIN',
        (modules, callback) => {
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
              expect(req.resource).toBe(undefined);
              expect(req.path).toBe(
                path.join(import.meta.dirname, '/index.js'),
              );
              expect(req.query).toBe('?query');
              callback();
            },
          );
        },
      );
    });
  }
}

/** @type {import("@rspack/core").Configuration} */
export default {
  entry: './index.js',
  plugins: [new Plugin()],
};
