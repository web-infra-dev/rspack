const { strictEqual } = require('assert');

class TestPlugin {
  apply(compiler) {
    let identifiers = [];
    compiler.hooks.compilation.tap('TestPlugin', (compilation) => {
      identifiers = [];
      compilation.hooks.buildModule.tap('TestPlugin', (module) => {
        identifiers.push(module.identifier());
      });
    });
    compiler.hooks.done.tap('TestPlugin', () => {
      const fooBuilds = identifiers.filter((identifier) =>
        identifier.endsWith('foo.js'),
      ).length;

      strictEqual(fooBuilds, 0);
    });
  }
}

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  context: __dirname,
  optimization: {
    sideEffects: true,
  },
  module: {
    rules: [
      {
        test: /foo\.js$/,
        sideEffects: false,
      },
    ],
  },
  plugins: [new TestPlugin()],
};
