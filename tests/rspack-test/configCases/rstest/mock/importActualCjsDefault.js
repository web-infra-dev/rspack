import mocked from './src/cjs-default.js';
import actual from './src/cjs-default.js' with { rstest: 'importActual' };
import { rstest } from '@rstest/core';

rstest.mock('./src/cjs-default.js', () => ({
  default: { ...actual, value: `mocked_${actual.value}` },
}));

it('importActual should read a CommonJS default import inside the mock factory', () => {
  expect(mocked.value).toBe('mocked_cjs');
  expect(actual.value).toBe('cjs');
});
