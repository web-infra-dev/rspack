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
  module: {
    parser: { javascript: { importMetaResolve: true } },
    rules: [{ test: /async\.txt$/, type: 'asset/resource' }],
  },
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
          compilation.hooks.optimizeTree.tap(
            { name: 'CheckResolveAssetChunks', stage: -1000 },
            () => {
              // Assets do not require standalone entry chunks.
              expect([...compilation.chunks]).toHaveLength(includeUsed ? 4 : 1);
            },
          );
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
            expect(chunks).toHaveLength(includeUsed ? 1 : 0);
            expect(chunks.some((chunk) => chunk.name === 'unused')).toBe(false);
            expect(Boolean(compilation.getAsset('asset.txt'))).toBe(
              includeUsed,
            );
            expect([...compilation.chunks]).toHaveLength(includeUsed ? 4 : 1);
            if (includeUsed) {
              // The lazy chunk can reuse the asset already available in its parent.
              expect(chunks[0].name).toBe('used');
              const asyncAsset = [...compilation.modules].find(
                (module) => module.rawRequest === './async.txt',
              );
              const asyncChunks = [
                ...chunkGraph.getModuleChunksIterable(asyncAsset),
              ];
              expect(asyncChunks).toHaveLength(1);
              expect(asyncChunks[0].hasRuntime()).toBe(false);
              expect(asyncChunks[0]).not.toBe(chunks[0]);
            }
          });
        },
      );
    },
  ],
}));
