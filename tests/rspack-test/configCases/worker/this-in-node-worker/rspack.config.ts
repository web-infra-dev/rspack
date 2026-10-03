import { defineConfig } from '@rspack/cli';

export default defineConfig(
  (['node14', 'async-node14'] as const).map((target) => ({
    target,
    module: {
      parser: {
        javascript: {
          worker: ['...', 'Worker from node:worker_threads'],
        },
      },
    },
  })),
);
