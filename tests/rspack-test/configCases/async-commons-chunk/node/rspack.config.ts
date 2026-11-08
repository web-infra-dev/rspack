import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'none',
  entry: {
    'foo/bar': './',
  },
  target: 'node',
  optimization: {
    chunkIds: 'named',
    moduleIds: 'named',
  },
});
