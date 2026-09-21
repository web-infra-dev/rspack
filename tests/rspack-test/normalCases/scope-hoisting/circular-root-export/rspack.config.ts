import { defineConfig } from '@rspack/cli';

export default defineConfig({
  optimization: {
    // Avoid the default export of root.js is inlined into external.js,
    // which causes the side effect of root.js not executed.
    inlineExports: false,
  },
});
