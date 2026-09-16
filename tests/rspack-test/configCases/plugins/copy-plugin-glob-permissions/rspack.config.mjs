import { CopyRspackPlugin } from '@rspack/core';
import path from 'node:path';
export default {
  entry: './index.js',
  target: 'node',
  plugins: [
    new CopyRspackPlugin({
      patterns: [
        {
          from: path.join(import.meta.dirname, 'src', '*.txt'),
          to: path.join(import.meta.dirname, 'dist'),
          copyPermissions: true,
        },
      ],
    }),
  ],
};
