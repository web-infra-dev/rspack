import { defineConfig } from '@rspack/cli';

export default defineConfig({
  experiments: {
    runtimeMode: 'rspack',
  },
  output: {
    library: {
      name: 'RuntimeModeLibraryExport',
      type: 'umd',
      export: 'default',
    },
  },
});
