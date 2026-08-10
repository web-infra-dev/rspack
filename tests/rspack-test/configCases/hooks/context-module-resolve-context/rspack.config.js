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
            if (
              resolveData.request.includes('after-source') ||
              resolveData.request.includes('after-resource-override')
            ) {
              return;
            }
            if (resolveData.request.includes('request-override')) {
              resolveData.context = path.join(__dirname, 'fixtures');
              resolveData.request = '../src/request-override';
              resolveData.recursive = true;
              return;
            }
            if (resolveData.request.includes('../shared')) {
              resolveData.context = path.join(__dirname, 'fixtures', 'nested');
              resolveData.recursive = false;
              return;
            }
            resolveData.context = path.join(__dirname, 'fixtures');
          },
        );
        contextModuleFactory.hooks.beforeResolve.tap(
          `${pluginName}-observer`,
          (resolveData) => {
            if (resolveData.request.includes('../src/request-override')) {
              assert.strictEqual(
                resolveData.context,
                path.join(__dirname, 'fixtures'),
              );
              assert.strictEqual(resolveData.recursive, true);
              return;
            }
            if (resolveData.request.includes('../shared')) {
              assert.strictEqual(
                resolveData.context,
                path.join(__dirname, 'fixtures', 'nested'),
              );
              assert.strictEqual(resolveData.recursive, false);
            }
          },
        );
        contextModuleFactory.hooks.afterResolve.tap(
          pluginName,
          (resolveData) => {
            if (resolveData.request.includes('after-resource-override')) {
              assert.strictEqual(
                resolveData.context,
                path.join(__dirname, 'src'),
              );
              resolveData.resource = path.join(
                __dirname,
                'fixtures',
                'after-resource',
              );
              return;
            }
            if (resolveData.request.includes('after-source')) {
              assert.strictEqual(
                resolveData.context,
                path.join(__dirname, 'src'),
              );
              resolveData.context = path.join(__dirname, 'fixtures');
              return;
            }
            if (resolveData.request.includes('../src/request-override')) {
              assert.strictEqual(
                resolveData.context,
                path.join(__dirname, 'fixtures'),
              );
              assert.strictEqual(resolveData.recursive, true);
              return;
            }
            if (resolveData.request.includes('../shared')) {
              assert.strictEqual(
                resolveData.context,
                path.join(__dirname, 'fixtures', 'nested'),
              );
              assert.strictEqual(resolveData.recursive, false);
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
