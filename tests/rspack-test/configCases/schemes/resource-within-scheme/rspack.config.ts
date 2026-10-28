import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: ['data:text/javascript,import "polyfill";', './index.js'],
});
