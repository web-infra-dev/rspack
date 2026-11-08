import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: ['node', 'es2020'],
  output: {
    environment: {
      // Our target supports `globalThis`, but for test purposes we set it to `false`
      globalThis: false,
    },
  },
});
