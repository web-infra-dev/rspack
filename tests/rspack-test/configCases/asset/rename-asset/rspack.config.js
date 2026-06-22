class Plugin {
  apply(compiler) {
    compiler.hooks.compilation.tap('Test', (compilation) => {
      compilation.hooks.processAssets.tap(
        {
          name: 'Test',
          stage: -100,
        },
        () => {
          compilation.renameAsset('chunk.js', 'renamed.js');
        },
      );
    });
  }
}

/**@type {import('@rspack/core').Configuration}*/
module.exports = {
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  context: __dirname,
  output: {
    chunkFilename: 'chunk.js',
  },
  plugins: [new Plugin()],
};
