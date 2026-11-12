import { defineConfig } from '@rspack/cli';
import createTestCases from '../_helpers/createTestCases.ts';

export default defineConfig(
  createTestCases({
    nothing: {
      usedExports: [],
      expect: {
        './components/Button': [],
        './components/ButtonGroup': [],
        './theme': [],
      },
    },
    all: {
      usedExports: ['default', 'ButtonGroup', 'themeNamespace'],
      expect: {
        './components/Button': ['default'],
        './components/ButtonGroup': ['default'],
        './theme': ['themeNamespace'],
      },
    },
  }),
);
