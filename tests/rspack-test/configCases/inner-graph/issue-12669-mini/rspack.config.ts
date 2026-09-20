import { defineConfig } from '@rspack/cli';
import createTestCases from '../_helpers/createTestCases.ts';

export default defineConfig(
  createTestCases({
    // nothing: {
    // 	usedExports: [],
    // 	expect: {
    // 		"./dependency": []
    // 	}
    // },
    a: {
      usedExports: ['a'],
      expect: {
        './dependency': ['x'],
      },
    },
    // b: {
    // 	usedExports: ["b"],
    // 	expect: {
    // 		"./dependency": ["y"]
    // 	}
    // }
  }),
);
