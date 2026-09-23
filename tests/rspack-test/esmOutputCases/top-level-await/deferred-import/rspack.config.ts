import { defineConfig } from '@rspack/cli';

export default defineConfig({
  experiments: { deferImport: true },
});
