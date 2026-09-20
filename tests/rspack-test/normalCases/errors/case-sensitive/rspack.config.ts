import { defineConfig } from '@rspack/cli';

import { CaseSensitivePlugin } from '@rspack/core';
export default defineConfig({
  plugins: [new CaseSensitivePlugin()],
});
