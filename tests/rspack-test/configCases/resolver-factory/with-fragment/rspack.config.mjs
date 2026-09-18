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
          // With fragment
          normalResolver.resolve(
            {},
            import.meta.dirname,
            './index.js#fragment',
            {},
            (error, res, req) => {
              expect(
                normalResolver.resolveSync(
                  {},
                  import.meta.dirname,
                  './index.js#fragment',
                ),
              ).toBe(res);

              expect(error).toBeNull();
              expect(res).toBe(
                path.join(import.meta.dirname, '/index.js#fragment'),
              );
              // webpack does not have resource field
              expect(req.resource).toBe(undefined);
              expect(req.path).toBe(
                path.join(import.meta.dirname, '/index.js'),
              );
              expect(req.fragment).toBe('#fragment');
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
