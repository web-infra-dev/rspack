import { defineConfig } from '../../../../../dist/index.js';

export default defineConfig({
  output: {
    filename: 'override.bundle.js',
  },
  name: 'override_config',
  mode: 'development',
});
