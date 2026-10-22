import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    library: {
      type: 'modern-module',
    },
  },
  optimization: {
    runtimeChunk: false,
    avoidEntryIife: true,
  },
});
