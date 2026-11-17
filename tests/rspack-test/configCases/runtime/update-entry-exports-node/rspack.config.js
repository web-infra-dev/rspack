module.exports = ['commonjs2', 'commonjs-module'].flatMap((type) =>
  [false, true].flatMap((enabled) =>
    ['node', 'async-node'].map((target) => ({
      target,
      entry: { main: './index.js', entry: './entry.js' },
      output: {
        filename: (data) =>
          data.chunk.name === 'main' ? 'bundle0.js' : '[name].js',
        library: { type },
      },
      optimization: {
        minimize: true,
        splitChunks: {
          chunks: 'all',
          minSize: 0,
          cacheGroups: {
            common: { test: /shared\.js$/, name: 'shared', enforce: true },
          },
        },
      },
      plugins: [
        (compiler) => {
          const { RuntimeGlobals, DefinePlugin } = compiler.webpack;
          new DefinePlugin({
            __TEST_ENABLED__: JSON.stringify(enabled),
            __TEST_RUNTIME__:
              compiler.options.experiments.runtimeMode === 'rspack'
                ? RuntimeGlobals.requireScope
                : RuntimeGlobals.require,
          }).apply(compiler);
          compiler.hooks.thisCompilation.tap(
            'ReloadableEntry',
            (compilation) => {
              compilation.hooks.additionalTreeRuntimeRequirements.tap(
                'ReloadableEntry',
                (_chunk, requirements) => {
                  if (enabled)
                    requirements.add(RuntimeGlobals.updateEntryExports);
                  requirements.add(RuntimeGlobals.moduleCache);
                },
              );
            },
          );
        },
      ],
    })),
  ),
);
