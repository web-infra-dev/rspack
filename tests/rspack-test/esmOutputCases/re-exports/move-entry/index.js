// normal export
export const a = (() => 42)()

// inlined export
export const b = 1

// re-export star
export * from './lib'


it('should have correct exports', async () => {
  const { a, b, readFile } = await import(/* webpackIgnore: true */ './main.mjs')
  expect(a).toBe(42)
  expect(b).toBe(1)

  expect(typeof readFile).toBe('function')
})