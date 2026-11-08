import { defineConfig } from '@rspack/cli';

export default defineConfig({
  node: {
    __dirname: false,
    __filename: false,
  },
  mode: 'production',
  optimization: {
    mangleExports: 'size',
    inlineExports: false,
  },
});
