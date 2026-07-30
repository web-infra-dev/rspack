const assert = require('node:assert');
const path = require('node:path');

const pluginName = 'context-module-resolve-context';

class ContextModuleResolveContextPlugin {
  apply(compiler) {
    compiler.hooks.contextModuleFactory.tap(
      pluginName,
      (contextModuleFactory) => {
        contextModuleFactory.hooks.beforeResolve.tap(
          pluginName,
          (resolveData) => {
            assert.strictEqual(
              resolveData.context,
              path.join(__dirname, 'src'),
            );
            assert.strictEqual(
              path.isAbsolute(resolveData.request.split(/[?#]/, 1)[0]),
              false,
            );
            if (resolveData.request.includes('after-source')) {
              return;
            }
            resolveData.context = path.join(__dirname, 'fixtures');
          },
        );
        contextModuleFactory.hooks.afterResolve.tap(
          pluginName,
          (resolveData) => {
            if (resolveData.request.includes('after-source')) {
              assert.strictEqual(
                resolveData.context,
                path.join(__dirname, 'src'),
              );
              resolveData.context = path.join(__dirname, 'fixtures');
              return;
            }
            assert.strictEqual(
              resolveData.context,
              path.join(__dirname, 'fixtures'),
            );
          },
        );
      },
    );
  }
}

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  context: __dirname,
  entry: './src/index.js',
  plugins: [new ContextModuleResolveContextPlugin()],
};
