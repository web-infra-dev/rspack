import { withRspackConfig } from '@rstest/adapter-rspack';
import { defineConfig } from '@rstest/core';

// Docs: https://rstest.rs/config/
export default defineConfig({
  extends: withRspackConfig(),
  setupFiles: ['./tests/rstest.setup.js'],
});
