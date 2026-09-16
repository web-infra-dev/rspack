module.exports = {
  target: 'node',
  output: { library: { type: 'commonjs2' } },
  plugins: [
    (compiler) => {
      const { RuntimeGlobals, DefinePlugin, Compilation, sources } =
        compiler.webpack;
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
            requirements.add(RuntimeGlobals.moduleCache);
          },
        );
        compilation.hooks.processAssets.tap(
          {
            name: 'TestEntryExports',
            stage: Compilation.PROCESS_ASSETS_STAGE_REPORT,
          },
          () => {
            compilation.updateAsset(
              'bundle0.js',
              (source) =>
                new sources.ConcatSource(
                  source,
                  '\nglobalThis.__TEST_EXPORTED__ = module.exports;\n',
                ),
            );
          },
        );
      });
    },
  ],
};
