import path from 'node:path';
export default {
  // mode: "development" || "production",
  entry: {
    main: './example',
  },
  optimization: {
    runtimeChunk: true,
  },
  output: {
    path: path.join(import.meta.dirname, 'dist'),
    filename: '[name].chunkhash.js',
    chunkFilename: '[name].chunkhash.js',
  },
};
