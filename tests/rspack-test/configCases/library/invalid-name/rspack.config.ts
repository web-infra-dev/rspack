import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    library: ['123-hello world', 'hello world'],
  },
});
