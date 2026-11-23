import { defineConfig } from '@rspack/cli';

export default defineConfig({
  externals: { value: 'var 42', byLayer: undefined },
});
