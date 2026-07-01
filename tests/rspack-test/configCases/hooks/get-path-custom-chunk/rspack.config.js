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
          compilation.getPath('[name]-[chunkhash]-[contenthash]', {
            chunk: mainChunk,
            contentHashType: 'javascript',
          }),
        ).toBe(`${mainChunk.name}-${mainChunk.renderedHash}-${contentHash}`);

        expect(
          compilation.getPath('[name]', {
            chunk: {
              name: 'main',
            },
          }),
        ).toBe('[name]');
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
