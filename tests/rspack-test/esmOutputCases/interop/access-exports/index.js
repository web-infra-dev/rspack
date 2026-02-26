import value from './cjs'

it('should have correct runtime interop', () => {
  expect(value).toBe(42)
})