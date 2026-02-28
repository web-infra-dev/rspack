export * from './entry2'
export * from './entry3'

it('should have all exports', async () => {
  const {  entry3, entry2 } = await import(/*webpackIgnore: true*/'./main.mjs')

  expect(entry2).toBe('entry2')
  expect(entry3).toBe('entry3')
})