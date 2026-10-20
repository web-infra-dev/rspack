import { defineConfig } from '@rspack/cli';

export default defineConfig(
  [false, true].map((concatenateModules) => ({
    mode: 'production',
    target: 'async-node',
    optimization: { concatenateModules, mangleExports: true },
    experiments: { deferImport: true },
  })),
);
