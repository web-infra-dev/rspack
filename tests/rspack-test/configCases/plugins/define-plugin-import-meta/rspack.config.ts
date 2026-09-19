import { defineConfig } from '@rspack/cli';

import { DefinePlugin } from '@rspack/core';

export default defineConfig({
  plugins: [
    new DefinePlugin({
      'import.meta.env.MODE': '"production"',
    }),
  ],
});
