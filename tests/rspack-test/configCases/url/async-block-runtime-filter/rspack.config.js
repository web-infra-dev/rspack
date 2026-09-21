module.exports = [true, false].map((includeUsed, index) => ({
  mode: 'production',
  devtool: false,
  target: 'web',
  entry: includeUsed
    ? { used: './used.js', unused: './unused.js' }
    : { unused: './unused.js' },
  output: {
    filename: `[name]-${index}.js`,
    chunkFilename: `[id]-${index}.js`,
    publicPath: '/path/',
    assetModuleFilename: '[name][ext]',
  },
  module: {
    rules: [
      { test: /target\.js$/, dependency: 'url', type: 'javascript/auto' },
    ],
  },
  optimization: {
    usedExports: true,
    innerGraph: true,
    concatenateModules: false,
    splitChunks: false,
    minimize: false,
  },
  plugins: [
    (compiler) => {
      compiler.hooks.compilation.tap('CheckUrlRuntimeFilter', (compilation) => {
        compilation.hooks.afterSeal.tap('CheckUrlRuntimeFilter', () => {
          const issuer = [...compilation.modules].find((m) =>
            m.resource?.endsWith('/shared.js'),
          );
          const { moduleGraph } = compilation;
          if (includeUsed) {
            expect([...moduleGraph.getUsedExports(issuer, 'used')]).toEqual([
              'loadAsset',
            ]);
          }
          expect([...moduleGraph.getUsedExports(issuer, 'unused')]).toEqual([
            'value',
          ]);
          expect(issuer.blocks).toHaveLength(1);
          const outer = issuer.blocks[0];
          expect(outer.blocks).toHaveLength(1);
          for (const block of [outer, outer.blocks[0]]) {
            const dependencies = block.dependencies.filter(
              (d) => d.type === 'new URL()',
            );
            expect(dependencies).toHaveLength(1);
            const dependency = dependencies[0];
            const connection = moduleGraph.getConnection(dependency);
            if (includeUsed) {
              expect(connection.getActiveState('used')).toBe(true);
            }
            expect(connection.getActiveState('unused')).toBe(false);
            const asset = moduleGraph.getModule(dependency);
            expect(
              moduleGraph.getExportsInfo(asset).isModuleUsed('unused'),
            ).toBe(false);
          }
          const nested = outer.blocks[0];
          expect(nested.blocks).toHaveLength(1);
          const entry = nested.blocks[0];
          expect(entry.dependencies).toHaveLength(1);
          const entryConnection = moduleGraph.getConnection(
            entry.dependencies[0],
          );
          // The promoted entry has its own runtime, so it uses the issuer's usage
          // across runtimes instead of treating that new runtime as unused.
          expect(entryConnection.getActiveState('unused')).toBe(includeUsed);
          for (const name of ['asset.txt', 'nested.txt']) {
            expect(Boolean(compilation.getAsset(name))).toBe(includeUsed);
          }
        });
      });
    },
  ],
}));
