export default {
  mode: 'development',
  entry: {
    main: './index.js',
    worker: './lib.js',
  },
  optimization: {
    runtimeChunk: false,
  },
};
