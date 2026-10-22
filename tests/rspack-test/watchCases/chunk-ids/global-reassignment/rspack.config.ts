import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: ['./index.js', './trigger.js'],
  cache: { type: 'memory' },
  optimization: {
    moduleIds: 'named',
    chunkIds: 'deterministic',
    concatenateModules: false,
    inlineExports: false,
  },
  incremental: true,
});
