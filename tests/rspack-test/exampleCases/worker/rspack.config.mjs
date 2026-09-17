import path from 'node:path';
export default {
  entry: './example.js',
  output: {
    path: path.join(import.meta.dirname, 'dist'),
    filename: '[name].js',
    chunkFilename: '[name].js',
    publicPath: '/dist/',
  },
  optimization: {
    concatenateModules: true,
    usedExports: true,
    providedExports: true,
    chunkIds: 'deterministic', // To keep filename consistent between different modes (for example building only)
  },
};
