import { "a.b" as lib } from './lib';

console.log.bind(lib)

export {lib as "b.c"}

it('should have correct export', async () => {
  const { "b.c": foo } = await import(/* webpackIgnore: true */ './main.mjs')

  expect(foo()).toBe(42)
})