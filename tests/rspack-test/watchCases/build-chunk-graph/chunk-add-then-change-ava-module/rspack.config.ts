import { defineConfig } from '@rspack/cli';

export default defineConfig({
  externals: {
    '@rspack/test-tools': 'commonjs @rspack/test-tools',
  },
  optimization: {
    splitChunks: false,
    sideEffects: false,
  },
  incremental: {
    buildChunkGraph: true,
  },
});
