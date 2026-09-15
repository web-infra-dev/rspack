import * as otherActual from './src/transitive/other' with { rstest: 'importActual' };
import { read } from './src/transitive/mid';
import { rstest } from '@rstest/core';

rstest.mock('./src/transitive/dep', () => {
  return { flag: () => 'MOCK' };
});

it('mock should apply to transitive deps reachable from an importActual import', () => {
  // The importActual target itself is the actual module...
  expect(otherActual.tag).toBe('REAL_OTHER');
  // ...but its transitive deps still follow mocks (shared module graph).
  expect(otherActual.viaOther()).toBe('MOCK');
  expect(read()).toBe('MOCK');
});
