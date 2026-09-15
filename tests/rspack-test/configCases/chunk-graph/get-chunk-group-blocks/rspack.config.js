class Plugin {
  apply(compiler) {
    compiler.hooks.compilation.tap('Test', (compilation) => {
      compilation.hooks.processAssets.tap('Test', () => {
        const { chunkGraph, moduleGraph } = compilation;
        const entry = compilation.entries.get('main');
        const entryModule = moduleGraph.getModule(entry.dependencies[0]);

        const blocks = entryModule.blocks;
        expect(blocks.length).toBe(3);

        // The first two `import()` calls share a chunk name, so they end up in
        // one group.
        const chunkGroup = chunkGraph.getBlockChunkGroup(blocks[0]);
        expect(chunkGroup.name).toBe('shared');
        expect(chunkGraph.getBlockChunkGroup(blocks[1])).toBe(chunkGroup);

        // Only the blocks of the requested group come back, not every block.
        const groupBlocks = chunkGraph.getChunkGroupBlocks(chunkGroup);
        expect(groupBlocks.length).toBe(2);

        // Blocks come back as the same JS instances the module exposes, and each
        // one still points back at the group it was read from.
        for (const block of groupBlocks) {
          expect(blocks).toContain(block);
          expect(chunkGraph.getBlockChunkGroup(block)).toBe(chunkGroup);
        }

        // Reaching the imported module from the group is the point of the API.
        const requests = groupBlocks.map(
          (block) => block.dependencies[0].request,
        );
        expect(requests.slice().sort()).toEqual(['./bar', './foo']);

        const lonelyGroup = chunkGraph.getBlockChunkGroup(blocks[2]);
        expect(lonelyGroup.name).toBe('lonely');
        expect(chunkGraph.getChunkGroupBlocks(lonelyGroup)).toEqual([
          blocks[2],
        ]);

        // The initial entrypoint is not created by a block, so it has none.
        const entrypoint = compilation.entrypoints.get('main');
        expect(chunkGraph.getChunkGroupBlocks(entrypoint)).toEqual([]);
      });
    });
  }
}

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  target: 'web',
  node: false,
  entry: {
    main: './index.js',
  },
  output: {
    filename: '[name].js',
  },
  optimization: {
    sideEffects: false,
  },
  plugins: [new Plugin()],
};
