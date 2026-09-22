import { defineConfig } from '@rspack/cli';

const base = defineConfig({
  entry: {
    web: './web',
    webworker: {
      import: './webworker',
      chunkLoading: 'import-scripts',
    },
  },
  target: 'web',
});

export default defineConfig([
  {
    externals: {
      './chunk-0.js': 'commonjs ./chunk-0.js',
    },
    ...base,
    output: { ...base.output, filename: '[name]-0.js' },
  },
  {
    externals: {
      './chunk-0.js': 'commonjs ./chunk-0.js',
    },
    ...base,
    output: { ...base.output, filename: '[name]-1.js' },
  },
]);
