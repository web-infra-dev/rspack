import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    library: { type: 'system' },
  },
  externals: {
    external1: 'external1',
    external2: 'external2',
    external3: 'external3',
    external4: 'external4',
    external5: 'external5',
  },
});
