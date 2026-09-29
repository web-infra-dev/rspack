import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    module: true,
    library: {
      type: 'module',
    },
    enabledLibraryTypes: ['module', 'module'],
  },
  target: ['es2022'],
});
