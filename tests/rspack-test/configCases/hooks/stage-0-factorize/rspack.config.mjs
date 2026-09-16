import TestPlugin from '../stage-compilation/plugin.js';

/** @type {import("@rspack/core").Configuration} */
export default {
  externals: ['commonjs fs', 'commonjs path'],
  plugins: [
    new TestPlugin((compiler, list) => {
      compiler.hooks.compilation.tap(
        TestPlugin.name,
        (compilation, { normalModuleFactory }) => {
          normalModuleFactory.hooks.factorize.tap(
            TestPlugin.name,
            (resolveData) => {
              list.push(`/*: ${resolveData.request} :*/`);
            },
          );
        },
      );
    }),
  ],
};
