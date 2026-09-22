import { defineConfig } from '@rspack/cli';

export default defineConfig({
  optimization: {
    // inline const will ignore TDZ
    inlineExports: false,
  },
});
