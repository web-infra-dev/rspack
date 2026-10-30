import { defineConfig } from '@rspack/cli';

// @ts-expect-error Rspack does not support the parallelism option yet.
export default defineConfig({
  parallelism: 1,
});
