import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    library: { type: 'assign-properties', name: ['process', 'env'] },
  },
});
