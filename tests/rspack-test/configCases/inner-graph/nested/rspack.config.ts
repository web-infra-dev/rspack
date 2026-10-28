import { defineConfig } from '@rspack/cli';
import createTestCases from '../_helpers/createTestCases.ts';

export default defineConfig(
  createTestCases({
    nothing: {
      usedExports: [],
      expect: {
        './assert': [],
      },
    },
    fun5: {
      usedExports: ['fun5'],
      expect: {
        './assert': ['deepEqual'],
      },
    },
    fun6: {
      usedExports: ['fun6'],
      expect: {
        './assert': ['equal'],
      },
    },
    all: {
      usedExports: ['fun5', 'fun6'],
      expect: {
        './assert': ['deepEqual', 'equal'],
      },
    },
  }),
);
