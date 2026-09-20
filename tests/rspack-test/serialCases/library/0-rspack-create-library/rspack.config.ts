import { defineConfig } from '@rspack/cli';

export default defineConfig(() => [
  {
    entry: {
      main: './modern-module-non-entry-module-export/index.js',
    },
    output: {
      module: true,
      chunkFormat: 'module',
      filename: 'modern-module-non-entry-module-export/[name].js',
      library: {
        type: 'modern-module',
      },
    },
    optimization: {
      concatenateModules: true,
      avoidEntryIife: true,
    },
  },
]);
