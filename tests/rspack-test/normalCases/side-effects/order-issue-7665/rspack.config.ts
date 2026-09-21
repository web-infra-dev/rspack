import { defineConfig } from '@rspack/cli';

export default defineConfig({
  optimization: {
    // Avoid the default export of module/b.js is inlined into module/index.js,
    // which causes the side effect of module/b.js not executed.
    inlineExports: false,
  },
});
