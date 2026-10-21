import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    filename: ({ chunk }) =>
      chunk?.name === 'runtime'
        ? 'bootstrap/nested/[name].js'
        : 'server/[name].js',
    chunkFilename: 'chunks/[name].js',
    chunkLoading: 'async-node',
    hotUpdateMainFilename:
      'updates/manifests/[runtime].[fullhash].hot-update.json',
    hotUpdateChunkFilename: 'updates/chunks/[id].[fullhash].hot-update.js',
  },
  optimization: {
    runtimeChunk: 'single',
  },
});
