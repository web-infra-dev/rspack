import { defineConfig } from '@rspack/cli';
import createTestCases from '../_helpers/createTestCases.ts';

export default defineConfig(
  createTestCases({
    nothing: {
      usedExports: [],
      expect: {
        '@angular/core': ['ɵccf', 'ɵcrt', 'ɵdid', 'ɵeld', 'ɵted', 'ɵvid'],
        './app.component': ['AppComponent'],
      },
    },
    AppComponentNgFactory: {
      usedExports: ['AppComponentNgFactory'],
      expect: {
        '@angular/core': ['ɵccf', 'ɵcrt', 'ɵdid', 'ɵeld', 'ɵted', 'ɵvid'],
        './app.component': ['AppComponent'],
      },
    },
  }),
);
