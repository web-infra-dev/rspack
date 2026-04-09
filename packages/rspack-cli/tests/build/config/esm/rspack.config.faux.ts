import path from 'node:path';

export default {
  mode: 'production',
  entry: path.resolve(__dirname, 'main.ts'),
  output: {
    path: path.resolve(__dirname, 'dist'),
    filename: 'faux-ts.bundle.js',
  },
};
