const pluginName = 'plugin';

class Plugin {
  apply(compiler) {
    let called = false;
    compiler.hooks.compilation.tap(pluginName, (compilation) => {
      compilation.hooks.processAssets.tap(pluginName, () => {
        called = true;
        const mainChunk = Array.from(compilation.chunks).find(
          (chunk) => chunk.name === 'main',
        );
        expect(mainChunk).toBeDefined();
        const contentHash = mainChunk.contentHash.javascript.slice(0, 20);

        expect(
          compilation.getPath('[id]-[name]-[chunkhash]-[contenthash]', {
            chunk: mainChunk,
            contentHashType: 'javascript',
          }),
        ).toBe(
          `${mainChunk.id}-${mainChunk.name}-${mainChunk.renderedHash}-${contentHash}`,
        );

        expect(
          compilation.getPath('[name]-[chunkhash]', {
            chunk: mainChunk,
          }),
        ).toBe(`${mainChunk.name}-${mainChunk.renderedHash}`);

        const rawChunkPathData = {
          name: 'chunkname',
          id: 'chunkid',
          hash: 'chunkhash',
          contentHash: {
            javascript: 'contenthash',
          },
        };

        expect(
          compilation.getPath('[id]-[name]-[chunkhash]-[contenthash]', {
            chunk: rawChunkPathData,
            contentHashType: 'javascript',
          }),
        ).toBe('chunkid-chunkname-chunkhash-contenthash');

        expect(
          compilation.getPath('[name]-[chunkhash]-[contenthash]', {
            chunk: {
              id: 'chunkid',
              hash: 'chunkhash',
              contentHash: {
                javascript: 'contenthash',
              },
            },
            contentHashType: 'javascript',
          }),
        ).toBe('chunkid-chunkhash-contenthash');

        expect(
          compilation.getPath('[name]-[chunkhash]-[contenthash]', {
            chunk: {
              id: 'chunkid',
              hash: 'chunkhash',
              contentHash: 'contenthash',
            },
          }),
        ).toBe('chunkid-chunkhash-contenthash');

        expect(
          compilation.getPath('[id]-[name]', {
            chunk: {
              id: 42,
            },
          }),
        ).toBe('42-42');

        expect(
          compilation.getPath('[contenthash]', {
            contentHash: 'asset-contenthash',
            chunk: {
              contentHash: 'chunk-contenthash',
            },
          }),
        ).toBe('asset-contenthash');

        expect(
          compilation.getAssetPathWithInfo(
            'static/css/[name].[contenthash].css',
            {
              chunk: {
                name: 'style',
                hash: 'chunkhash',
                contentHash: 'contenthash',
              },
            },
          ).path,
        ).toBe('static/css/style.contenthash.css');

        expect(
          compilation.getAssetPathWithInfo(
            'static/css/[name].[contenthash].css',
            {
              chunk: rawChunkPathData,
              contentHashType: 'javascript',
            },
          ).path,
        ).toBe('static/css/chunkname.contenthash.css');
      });
    });
    compiler.hooks.done.tap(pluginName, (stats) => {
      let json = stats.toJson();
      expect(json.errors.length === 0);
      expect(called).toBe(true);
    });
  }
}

/**@type {import("@rspack/core").Configuration}*/
module.exports = {
  context: __dirname,
  plugins: [new Plugin()],
};
