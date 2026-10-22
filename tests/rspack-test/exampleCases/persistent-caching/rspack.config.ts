import path from 'node:path';
import { defineConfig } from '@rspack/cli';

export default defineConfig({
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
  cache: {
    type: 'persistent',
    buildDependencies: [import.meta.filename],
    storage: {
      type: 'filesystem',
      location: path.resolve(import.meta.dirname, '.cache'),
    },
  },
});
