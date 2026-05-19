class Plugin {
  apply(compiler) {
    compiler.hooks.compilation.tap('Test', (compilation) => {
      compilation.hooks.processAssets.tap('Test', () => {
        const sharedModule = Array.from(compilation.modules).find((module) =>
          module.resource?.endsWith('shared.js'),
        );

        expect(sharedModule).toBeTruthy();

        const hashA = compilation.chunkGraph.getModuleHash(sharedModule, 'a');
        const hashB = compilation.chunkGraph.getModuleHash(sharedModule, 'b');

        expect(hashA).toBeTruthy();
        expect(hashB).toBeTruthy();
        expect(hashA).not.toEqual(hashB);
      });
    });
  }
}

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'production',
  entry: {
    a: './a.js',
    b: './b.js',
  },
  output: {
    filename: '[name].js',
  },
  optimization: {
    chunkIds: 'named',
    concatenateModules: false,
    minimize: false,
    moduleIds: 'named',
    usedExports: true,
  },
  plugins: [new Plugin()],
};
