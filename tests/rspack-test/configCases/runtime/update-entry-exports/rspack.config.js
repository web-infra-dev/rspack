const { RuntimeModule } = require('@rspack/core');

module.exports = [false, true].map((startup) => ({
  target: 'node',
  entry: ['./first.js', './index.js'],
  optimization: { minimize: false },
  plugins: [
    (compiler) => {
      const { RuntimeGlobals, DefinePlugin } = compiler.webpack;
      new DefinePlugin({
        __TEST_RUNTIME__:
          compiler.options.experiments.runtimeMode === 'rspack'
            ? RuntimeGlobals.requireScope
            : RuntimeGlobals.require,
      }).apply(compiler);
      compiler.hooks.thisCompilation.tap('TestEntryExports', (compilation) => {
        compilation.hooks.additionalTreeRuntimeRequirements.tap(
          'TestEntryExports',
          (chunk, requirements) => {
            requirements.add(RuntimeGlobals.updateEntryExports);
            requirements.add(RuntimeGlobals.moduleCache);
            if (startup) requirements.add(RuntimeGlobals.startup);
            class InspectStartup extends RuntimeModule {
              generate() {
                return '__webpack_require__.readStartupExports = function() { return __webpack_exports__; };';
              }
            }
            if (compiler.options.experiments.runtimeMode !== 'rspack') {
              compilation.addRuntimeModule(
                chunk,
                new InspectStartup('inspect startup'),
              );
            }
          },
        );
      });
    },
  ],
}));
