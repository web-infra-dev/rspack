module.exports = {
  target: 'node',
  output: { filename: '[name].js' },
  optimization: { runtimeChunk: 'single' },
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
          (_chunk, requirements) => {
            requirements.add(RuntimeGlobals.updateEntryExports);
          },
        );
      });
    },
  ],
};
