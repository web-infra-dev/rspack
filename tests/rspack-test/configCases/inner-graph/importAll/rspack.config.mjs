import createTestCases from '../_helpers/createTestCases.mjs';
export default createTestCases({
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
});
