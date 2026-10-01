import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    library: {
      type: 'jsonp',
      name: 'MyJsonpCallback',
    },
  },
});
