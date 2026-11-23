import { defineConfig } from '@rspack/cli';

export default defineConfig(
  [true, false].map((logicalAssignment, index) => ({
    target: 'web',
    output: {
      filename: `bundle${index}.js`,
      environment: {
        logicalAssignment,
      },
    },
  })),
);
