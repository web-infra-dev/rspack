import { v } from './a'

import { shared } from './shared'

it('should merge dependencies', async () => {
  const { v, shared } = await import(/* webpackIgnore: true */ './main.mjs')
  expect(v()).toBe(1)
  expect(shared()).toBe(42)
})

export { v, shared }
