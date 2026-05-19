it('should have no other exports', async () => {
  const { value } = await import('./lib')

  const ns = await import(/*webpackIgnore: true*/ './main.mjs')

  expect(value()).toBe(42)
  expect(Reflect.ownKeys(ns).length).toBe(1)
})