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
export default {
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  context: import.meta.dirname,
  output: {
    chunkFilename: 'chunk.js',
  },
  plugins: [new Plugin()],
};
