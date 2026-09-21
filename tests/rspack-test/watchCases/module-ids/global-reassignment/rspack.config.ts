import { defineConfig } from '@rspack/cli';

export default defineConfig({
  cache: { type: 'memory' },
  optimization: {
    moduleIds: 'deterministic',
    chunkIds: 'named',
    concatenateModules: false,
    inlineExports: false,
  },
  incremental: true,
});
