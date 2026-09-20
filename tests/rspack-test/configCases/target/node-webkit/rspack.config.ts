import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: [
    'nwjs',
    'nwjs0',
    'nwjs0.80',
    'node-webkit',
    'node-webkit0',
    'node-webkit0.80',
  ],
});
