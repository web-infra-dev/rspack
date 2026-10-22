import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'node',
  output: {
    library: { type: 'commonjs-static' },
  },
});
