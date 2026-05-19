const { join, normalize } = require('path');

const PLUGIN_NAME = 'Test';

class Plugin {
  /**
   * @param {import("@rspack/core").Compiler} compiler
   */
  apply(compiler) {
    compiler.hooks.compilation.tap(PLUGIN_NAME, (compilation) => {
      compilation.hooks.optimizeModules.tap(PLUGIN_NAME, () => {
        const moduleA = Array.from(compilation.modules).find(
          (module) =>
            normalize(module.resource) === normalize(join(__dirname, 'a.js')),
        );
        expect(compilation.moduleGraph.getUsedExports(moduleA, 'main')).toEqual(
          ['good'],
        );
        expect(
          compilation.moduleGraph.getUsedExports(moduleA, ['main']),
        ).toEqual(['good']);

        const moduleB = Array.from(compilation.modules).find(
          (module) =>
            normalize(module.resource) === normalize(join(__dirname, 'b.js')),
        );
        expect(compilation.moduleGraph.getUsedExports(moduleB, 'main')).toBe(
          false,
        );
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
    usedExports: true,
  },
  plugins: [new Plugin()],
};
