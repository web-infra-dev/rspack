module.exports = {
  cache: false,
  experiments: {
    newCache: {
      module: true,
      codeGeneration: false,
      loader: false,
      devtool: false,
      minimize: false,
    },
  },
  plugins: [
    {
      name: 'CountBuildsPlugin',
      builds: 0,
      apply(compiler) {
        compiler.hooks.done.tap(this.name, () => this.builds++);
      },
    },
  ],
};
