module.exports = {
  optimization: {
    moduleIds: 'named',
    chunkIds: 'named',
    usedExports: false,
    concatenateModules: false,
    splitChunks: {
      cacheGroups: {
        default: false,
        defaultVendors: false,
        shared: {
          test: /[\\/]dep\.js$/,
          name: 'shared',
          enforce: true,
        },
      },
    },
  },
  plugins: [
    (compiler) => {
      let build = 0;
      compiler.hooks.thisCompilation.tap('MoveUnchangedFactory', () => {
        build++;
      });
      // The first rebuild moves the existing page factory into an unchanged
      // shared chunk. It must be completed despite having no ordinary delta.
      compiler.options.optimization.splitChunks.cacheGroups.shared.chunks = (
        chunk,
      ) => chunk.name === 'other' || (build > 1 && chunk.name === 'page');
    },
  ],
};
