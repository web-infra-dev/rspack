import { defineConfig } from '@rspack/cli';

import { DefinePlugin } from '@rspack/core';

export default defineConfig({
  target: 'node',
  mode: 'production',
  plugins: [
    new DefinePlugin({
      'process.env.ENVIRONMENT': JSON.stringify('node'),
    }),
  ],
});
