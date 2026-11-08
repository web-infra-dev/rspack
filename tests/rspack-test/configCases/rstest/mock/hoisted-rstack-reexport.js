import { rs } from './reexport-rstack-test';

const mocks = rs.hoisted(() => ({
  mockedFn: rs.fn(),
}));

it('rs.hoisted should work with rs.fn re-exported by a user module', () => {
  expect(typeof mocks.mockedFn).toBe('function');
});
