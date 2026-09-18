import path from 'node:path';
export default {
  mode: 'development',
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'javascript/auto',
        use: ['style-loader', 'css-loader'],
      },
    ],
  },
  experiments: {
    cache: {
      type: 'persistent',
      buildDependencies: [import.meta.filename],
      storage: {
        type: 'filesystem',
        location: path.resolve(import.meta.dirname, '.cache'),
      },
    },
  },
};
