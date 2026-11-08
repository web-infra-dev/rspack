import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    library: {
      name: 'MyLibraryProperties',
      type: 'assign-properties',
    },
  },
});
