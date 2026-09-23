module.exports = [true, false].map((includeUsed, index) => ({
  mode: 'production',
  target: 'web',
  entry: includeUsed
    ? { used: './used.js', unused: './unused.js' }
    : { unused: './unused.js' },
  output: {
    filename: `[name]-${index}.js`,
    chunkFilename: `[id]-${index}.js`,
    assetModuleFilename: '[name][ext]',
    publicPath: '/path/',
  },
  module: { parser: { javascript: { importMetaResolve: true } } },
  optimization: {
    usedExports: true,
    innerGraph: true,
    concatenateModules: false,
    minimize: false,
    splitChunks: false,
  },
  plugins: [
    (compiler) => {
      compiler.hooks.compilation.tap(
        'CheckResolveAssetChunks',
        (compilation) => {
          compilation.hooks.afterSeal.tap('CheckResolveAssetChunks', () => {
            const { moduleGraph, chunkGraph } = compilation;
            const shared = [...compilation.modules].find(
              (module) => module.rawRequest === './shared.js',
            );
            const dependency = shared.blocks.find(
              (block) => block.dependencies[0]?.type === 'import.meta.resolve',
            ).dependencies[0];
            const connection = moduleGraph.getConnection(dependency);
            expect(connection.getActiveState('unused')).toBe(false);
            const asset = moduleGraph.getModule(dependency);
            const chunks = [...chunkGraph.getModuleChunksIterable(asset)];
            expect(chunks).toHaveLength(includeUsed ? 2 : 0);
            expect(chunks.some((chunk) => chunk.name === 'unused')).toBe(false);
            expect(Boolean(compilation.getAsset('asset.txt'))).toBe(
              includeUsed,
            );
            expect([...compilation.chunks]).toHaveLength(includeUsed ? 3 : 1);
            if (includeUsed) {
              const lazy = [...compilation.modules].find(
                (module) => module.rawRequest === './lazy.js',
              );
              for (const chunk of chunkGraph.getModuleChunksIterable(lazy)) {
                expect(chunks).toContain(chunk);
              }
            }
          });
        },
      );
    },
  ],
}));
