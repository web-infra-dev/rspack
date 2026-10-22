import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    module: true,
    publicPath: '/public/',
  },
  target: 'web',
});
