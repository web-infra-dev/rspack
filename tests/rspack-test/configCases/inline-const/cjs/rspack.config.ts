import { defineConfig } from '@rspack/cli';

export default defineConfig({
  externals: {
    fs: 'node-commonjs fs',
  },
  entry: './index.cjs',
  optimization: {
    moduleIds: 'named',
    inlineExports: true,
  },
});
