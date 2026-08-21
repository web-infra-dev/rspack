const path = require('path');

class CheckModuleTypePlugin {
  /** @param {import('@rspack/core').Compiler} compiler */
  apply(compiler) {
    compiler.hooks.afterEmit.tap('CheckModuleTypePlugin', (compilation) => {
      const moduleTypes = Object.fromEntries(
        Array.from(compilation.modules)
          .filter((module) => module.resource?.endsWith('.ts'))
          .map((module) => [
            path.relative(__dirname, module.resource).replaceAll(path.sep, '/'),
            module.type,
          ]),
      );

      expect(moduleTypes).toEqual({
        'auto.ts': 'javascript/auto',
        'commonjs/index.ts': 'javascript/dynamic',
        'esm/index.ts': 'javascript/esm',
        'esm/value.ts': 'javascript/esm',
      });
    });
  }
}

/** @type {import('@rspack/core').Configuration} */
module.exports = {
  resolve: {
    extensions: ['.ts', '...'],
  },
  plugins: [new CheckModuleTypePlugin()],
};
