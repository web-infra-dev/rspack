import { defineConfig } from '@rspack/cli';

export default defineConfig({
  lazyCompilation: {
    entries: false,
    imports: false,
  },
});
