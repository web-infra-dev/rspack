import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    library: {
      name: 'MyLibrary',
      type: 'assign',
    },
  },
});
