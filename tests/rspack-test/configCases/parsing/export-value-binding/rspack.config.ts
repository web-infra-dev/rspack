import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  optimization: {
    concatenateModules: false,
    inlineExports: false,
    mangleExports: false,
    // Explicitly minify: value bindings are emitted via `.d(..., values)` and
    // must still install data descriptors after SWC minify.
    minimize: true,
  },
});
