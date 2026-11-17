import { defineConfig } from '@rspack/cli';

export default defineConfig({
  cache: { type: 'persistent', portable: true },
});
