const { join, normalize } = require('path');

const PLUGIN_NAME = 'Test';

class Plugin {
  /**
   * @param {import("@rspack/core").Compiler} compiler
   */
  apply(compiler) {
    compiler.hooks.compilation.tap(PLUGIN_NAME, (compilation) => {
      compilation.hooks.optimizeModules.tap(PLUGIN_NAME, () => {
        // ESM module with named exports
        const moduleA = Array.from(compilation.modules).find(
          (module) =>
            normalize(module.resource) === normalize(join(__dirname, 'a.js')),
        );
        expect(
          compilation.moduleGraph.getProvidedExports(moduleA).sort(),
        ).toEqual(['bar', 'foo']);

        // ESM module with no exports (side-effect only)
        const moduleB = Array.from(compilation.modules).find(
          (module) =>
            normalize(module.resource) === normalize(join(__dirname, 'b.js')),
        );
        expect(compilation.moduleGraph.getProvidedExports(moduleB)).toEqual([]);

        // ESM module with default export
        const moduleD = Array.from(compilation.modules).find(
          (module) =>
            normalize(module.resource) === normalize(join(__dirname, 'd.js')),
        );
        expect(compilation.moduleGraph.getProvidedExports(moduleD)).toEqual([
          'default',
        ]);

        // CJS module (all exports potentially provided)
        const moduleC = Array.from(compilation.modules).find(
          (module) =>
            normalize(module.resource) === normalize(join(__dirname, 'c.js')),
        );
        expect(compilation.moduleGraph.getProvidedExports(moduleC)).toBe(true);
      });
    });
  }
}

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  target: 'web',
  node: {
    __dirname: false,
    __filename: false,
  },
  optimization: {
    providedExports: true,
  },
  plugins: [new Plugin()],
};
