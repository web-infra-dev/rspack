import { defineConfig } from '@rspack/cli';
import createTestCases from '../_helpers/createTestCases.ts';

export default defineConfig(
  createTestCases({
    nothing: {
      usedExports: [],
      expect: {
        './assert': ['equal'],
      },
    },
    myFunction: {
      usedExports: ['myFunction'],
      expect: {
        './assert': ['deepEqual', 'equal'],
      },
    },
  }),
);
