import { defineConfig } from '@rspack/cli';
import createTestCases from '../_helpers/createTestCases.ts';

export default defineConfig(
  createTestCases({
    nothing: {
      usedExports: [],
      expect: {
        './internal/_curry2': [],
      },
    },
    all: {
      usedExports: ['default'],
      expect: {
        './internal/_curry2': ['default'],
      },
    },
  }),
);
