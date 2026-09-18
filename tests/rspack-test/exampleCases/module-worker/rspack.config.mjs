import path from 'node:path';
export default {
  entry: './example.js',
  output: {
    module: true,
    path: path.join(import.meta.dirname, 'dist'),
    filename: '[name].js',
    chunkFilename: '[name].js',
    publicPath: '/dist/',
  },
  optimization: {
    chunkIds: 'deterministic', // To keep filename consistent between different modes (for example building only)
  },
  target: 'browserslist: last 2 Chrome versions',
};
