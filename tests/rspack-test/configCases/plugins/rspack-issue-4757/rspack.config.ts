import { defineConfig } from '@rspack/cli';

import { SwcJsMinimizerRspackPlugin } from '@rspack/core';

export default defineConfig({
  plugins: [new SwcJsMinimizerRspackPlugin()],
});
