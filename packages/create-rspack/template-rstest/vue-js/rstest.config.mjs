import { pluginVue } from '@rsbuild/plugin-vue';
import { defineConfig } from '@rstest/core';

// Docs: https://rstest.rs/config/
export default defineConfig({
  plugins: [pluginVue()],
  testEnvironment: 'happy-dom',
  setupFiles: ['./tests/rstest.setup.js'],
});
