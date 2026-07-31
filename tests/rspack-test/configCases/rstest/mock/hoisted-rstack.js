import { rs } from 'rstack/test';

const mocks = rs.hoisted(() => ({
  mockedFn: rs.fn(),
}));

it('rs.hoisted should work with rs.fn from rstack/test', () => {
  expect(typeof mocks.mockedFn).toBe('function');
});
