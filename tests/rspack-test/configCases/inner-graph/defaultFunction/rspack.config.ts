import { defineConfig } from '@rspack/cli';
import createTestCases from '../_helpers/createTestCases.ts';

export default defineConfig(
  createTestCases({
    nothing: {
      usedExports: [],
      expect: {
        any: [],
      },
    },
    default: {
      usedExports: ['default'],
      expect: {
        any: ['fun1'],
      },
    },
  }),
);
