import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    chunkLoadingGlobal: '__LOADED_CHUNKS__',
  },
  target: 'web',
});
