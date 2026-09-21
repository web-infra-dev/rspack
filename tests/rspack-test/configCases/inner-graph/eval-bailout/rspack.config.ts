import { defineConfig } from '@rspack/cli';
import createTestCases from '../_helpers/createTestCases.ts';

export default defineConfig(
  createTestCases({
    nothing: {
      usedExports: [],
      expect: {
        './test': [],
      },
    },
    nonEval: {
      usedExports: ['x'],
      expect: {
        './test': ['a'],
      },
    },
    directEval: {
      usedExports: ['y'],
      expect: {
        './test': ['a', 'b', 'c'],
      },
    },
    indirectEval: {
      usedExports: ['z'],
      expect: {
        './test': ['a', 'b', 'c'],
      },
    },
  }),
);
