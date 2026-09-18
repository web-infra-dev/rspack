import path from 'node:path';

/** @type {import("@rspack/core").Configuration} */
export default {
  target: 'node',
  output: {
    filename: '[name].js',
  },
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.make.tap('make', (compilation) => {
          const childEntry = path.resolve(
            import.meta.dirname,
            './child-entry.js',
          );
          const childCompiler = compilation.createChildCompiler('name', {}, [
            new compiler.rspack.EntryPlugin(compiler.context, childEntry),
          ]);
          childCompiler.compile(() => {});
        });
      },
    },
  ],
  optimization: {
    splitChunks: {
      cacheGroups: {
        singleVendor: {
          chunks: 'all',
          enforce: true,
          name: 'vendor',
        },
      },
    },
  },
};
