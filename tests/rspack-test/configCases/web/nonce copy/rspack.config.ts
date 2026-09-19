import { defineConfig } from '@rspack/cli';

import { rspack } from '@rspack/core';

export default defineConfig({
  target: 'web',
  // plugin that intercepts __webpack_require__
  plugins: [new rspack.HotModuleReplacementPlugin()],
});
