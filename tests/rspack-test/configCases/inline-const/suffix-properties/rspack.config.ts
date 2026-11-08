import { defineConfig } from '@rspack/cli';

export default defineConfig({
  externals: {
    fs: 'node-commonjs fs',
  },
  optimization: {
    moduleIds: 'named',
    inlineExports: true,
  },
});
