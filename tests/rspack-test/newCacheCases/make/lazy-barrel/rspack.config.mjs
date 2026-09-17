import path from 'node:path';

/** @type {import("@rspack/core").Configuration} */
export default {
  context: import.meta.dirname,
  optimization: {
    // avoid analyze side effects that will change index.js dependencies at HMR
    sideEffects: 'flag',
  },
  cache: {
    type: 'persistent',
  },
  experiments: {
    lazyBarrel: true,
  },
  plugins: [
    function (compiler) {
      let createdModules = new Set();
      compiler.hooks.compilation.tap(
        'test',
        (compilation, { normalModuleFactory }) => {
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
    },
  ],
};
