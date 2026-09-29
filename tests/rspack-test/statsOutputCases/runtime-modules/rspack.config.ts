import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: './index',
  mode: 'development',
  stats: {
    all: false,
    modules: true,
    runtimeModules: true,
  },
});
