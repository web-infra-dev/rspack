import path from 'node:path';
export default {
  module: {
    rules: [
      {
        test: /\.js$/,
        loader: path.resolve(import.meta.dirname, './example-loader'),
      },
    ],
  },
  resolveLoader: {
    alias: {
      'import-module-example': path.resolve(
        import.meta.dirname,
        './import-module-example-loader',
      ),
    },
  },
};
