import { value } from './src/foo'
import * as actual from './src/foo' with { rstest: 'importActual' };
import { rstest } from '@rstest/core';

rstest.mock('./src/foo', () =>{
  return {
    ...actual,
    value: `mocked_${actual?.value}`,
  }
})

afterEach(() => {
  rstest.doUnmock('./src/foo')
})

it('importActual should work with hoisted import', async () => {
  expect(value).toBe('mocked_foo')
  expect(actual.value).toBe('foo')
})
