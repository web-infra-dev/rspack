import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    library: { type: 'system', name: 'named-system-module-[name]' },
  },
  node: {
    __dirname: false,
    __filename: false,
  },
});
