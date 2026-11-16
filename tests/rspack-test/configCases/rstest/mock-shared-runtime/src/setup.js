import { a } from './a';

const rstestScope = typeof __rspack_context === 'undefined' ? __webpack_require__ : __rspack_context;
rstestScope.rstest_mock('./src/a.js', () => ({ a: 2 }));

if (a !== 1) {
  throw new Error(`setup should import original value before mocking, got ${a}`);
}
