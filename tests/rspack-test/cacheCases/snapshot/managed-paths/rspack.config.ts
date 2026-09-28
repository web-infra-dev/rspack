import { defineConfig } from '@rspack/cli';
import path from 'node:path';

export default defineConfig({
  context: import.meta.dirname,
  cache: {
    type: 'persistent',
    snapshot: {
      managedPaths: [path.join(import.meta.dirname, './test_lib')],
    },
  },
});
