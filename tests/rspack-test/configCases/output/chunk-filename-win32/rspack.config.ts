import { defineConfig } from '@rspack/cli';

import path from 'node:path';

export default defineConfig({
  entry: './index.js',
  output: {
    chunkFilename: path.win32.join('./', 'js/[name].[chunkhash:8].chunk.js'),
  },
});
