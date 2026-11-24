import rs from './src/cjs-test-api.cjs';

const mocks = rs.hoisted(() => ({
  mockedFn: rs.fn(),
}));

it('rs.hoisted should work with rs from a CommonJS default import', () => {
  expect(typeof mocks.mockedFn).toBe('function');
});
