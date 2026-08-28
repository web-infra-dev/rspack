const assert = require('assert');

const PLUGIN_NAME = 'test-plugin';

class Plugin {
  apply(compiler) {
    const order = [];

    compiler.hooks.compilation.tap(PLUGIN_NAME, (compilation) => {
      compilation.hooks.afterSeal.tap(PLUGIN_NAME, () => {
        order.push('afterSeal');
      });
    });
    compiler.hooks.afterCompile.tapPromise(PLUGIN_NAME, async (compilation) => {
      // The compilation is sealed but nothing has been written yet, so plugins
      // can still fail the build here instead of after the assets landed.
      assert(Object.keys(compilation.assets).length > 0);
      order.push('afterCompile');
    });
    compiler.hooks.shouldEmit.tap(PLUGIN_NAME, () => {
      order.push('shouldEmit');
      return true;
    });
    compiler.hooks.emit.tap(PLUGIN_NAME, () => {
      order.push('emit');
    });
    compiler.hooks.afterEmit.tap(PLUGIN_NAME, () => {
      order.push('afterEmit');
    });

    compiler.hooks.done.tap(PLUGIN_NAME, () => {
      order.push('done');
      // Same order as webpack, see https://github.com/web-infra-dev/rspack/issues/15313
      assert.deepStrictEqual(order, [
        'afterSeal',
        'afterCompile',
        'shouldEmit',
        'emit',
        'afterEmit',
        'done',
      ]);
    });
  }
}

/**@type {import("@rspack/core").Configuration}*/
module.exports = {
  context: __dirname,
  plugins: [new Plugin()],
};
