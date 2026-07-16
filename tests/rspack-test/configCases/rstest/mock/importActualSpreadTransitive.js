import * as otherActual from './src/transitive/other' with { rstest: 'importActual' };
import { tag, viaOther } from './src/transitive/other';
import { read } from './src/transitive/mid';
import { rstest } from '@rstest/core';

rstest.mock('./src/transitive/other', () => {
  return { ...otherActual, tag: 'MOCKED_OTHER' };
});

rstest.mock('./src/transitive/dep', () => {
  return { flag: () => 'MOCK' };
});

it('spreading importActual into a mock factory should keep transitive mocks', () => {
  // The factory ran and observed the initialized importActual binding.
  expect(tag).toBe('MOCKED_OTHER');
  // The spread actual implementation still sees the mocked transitive dep.
  expect(viaOther()).toBe('MOCK');
  expect(read()).toBe('MOCK');
});
