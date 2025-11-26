import { rs } from '@rstest/core';

const mocks = rs.hoisted(() => {
	globalThis.rstestCore = {
  rs: {
    fn: function() {
      return function mockFn() {}
    }
  }
};

  return {
    mockedFn: rs.fn()
  }
})

it('rs.hoisted should work with rs.fn from @rstest/core', () => {
  expect(typeof mocks.mockedFn).toBe('function')
})
