import { defineConfig } from '@rspack/cli';
import createTestCases from '../_helpers/createTestCases.ts';

export default defineConfig(
  createTestCases({
    nothing: {
      usedExports: [],
      expect: {
        'lodash-es': [],
      },
    },
    all: {
      usedExports: ['default'],
      expect: {
        'lodash-es': ['uniq'],
      },
    },
  }),
);
