import obj from './cjs'

export { obj }

it('should have correct export', async () => {
  const { obj } = await import(/*webpackIgnore: true*/ './main.mjs')

  expect(obj).toHaveProperty('value')
  expect(obj.value).toBe(42)
})