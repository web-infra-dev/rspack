import path from 'node:path';

const PLUGIN_NAME = 'plugin';
const TEST_DIR = path.resolve(import.meta.dirname, './src/');

class Plugin {
  /**
   * @param {import("@rspack/core").Compiler} compiler
   */
  apply(compiler) {
    compiler.hooks.afterCompile.tap(PLUGIN_NAME, (compilation) => {
      compilation.contextDependencies.add(TEST_DIR);
      expect(compilation.contextDependencies.has(TEST_DIR)).toBeTruthy();
      expect(
        [...compilation.contextDependencies].includes(TEST_DIR),
      ).toBeTruthy();
    });
  }
}

/**@type {import("@rspack/core").Configuration}*/
export default {
  entry: './index.js',
  plugins: [new Plugin()],
};
