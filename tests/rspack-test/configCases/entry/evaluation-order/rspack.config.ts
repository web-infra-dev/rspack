import { defineConfig } from '@rspack/cli';

export default defineConfig({
  // target: "node",
  entry: ['./before.js', './index.js'],
});
