import { defineConfig } from '@rspack/cli';

export default defineConfig({
  resolve: Object.freeze({}),
  // this fails to compile when the object is not cloned
});
