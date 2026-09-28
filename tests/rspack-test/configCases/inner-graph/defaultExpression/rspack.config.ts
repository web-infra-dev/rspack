import { defineConfig } from '@rspack/cli';
import createTestCases from '../_helpers/createTestCases.ts';

export default defineConfig(
  createTestCases({
    nothing: {
      usedExports: [],
      expect: {
        any: ['fun2', 'var1'],
      },
    },
    all: {
      usedExports: ['default'],
      expect: {
        any: ['fun2', 'var1'],
      },
    },
  }),
);
