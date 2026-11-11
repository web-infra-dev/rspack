import { defineConfig } from '@rspack/cli';

export default defineConfig(
  (['webpack', 'rspack'] as const).map((runtimeMode) => ({
    experiments: { runtimeMode },
    optimization: { concatenateModules: false },
  })),
);
