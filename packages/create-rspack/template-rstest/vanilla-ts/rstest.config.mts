import { defineConfig } from '@rstest/core';

// Docs: https://rstest.rs/config/
export default defineConfig({
  testEnvironment: 'happy-dom',
  setupFiles: ['./tests/rstest.setup.ts'],
});
