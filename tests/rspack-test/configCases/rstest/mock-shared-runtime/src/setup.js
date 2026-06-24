import { a } from './a';

__webpack_require__.rstest_mock('./src/a.js', () => ({ a: 2 }));

if (a !== 1) {
  throw new Error(`setup should import original value before mocking, got ${a}`);
}
