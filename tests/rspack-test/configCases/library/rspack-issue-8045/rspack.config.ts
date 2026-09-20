import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    library: {
      name: 'MyLibrary',
      export: 'default',
      type: 'window',
    },
  },
});
