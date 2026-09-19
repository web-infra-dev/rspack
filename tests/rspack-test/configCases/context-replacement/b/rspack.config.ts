import { defineConfig } from '@rspack/cli';

import { rspack as webpack } from '@rspack/core';

export default defineConfig({
  plugins: [
    new webpack.ContextReplacementPlugin(/context-replacement.b$/, /^\.\/only/),
  ],
});
