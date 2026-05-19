export * from 'fs'
export * from 'path'

import * as runtime from './runtime-decide'

it('should have correct exports as its exports is decided at runtime', async () => {
  const { resolve } = await import(/* webpackIgnore: true */ './main.mjs');
  const { resolve: nodeResolve } = await import(/* webpackIgnore: true */ 'path');

  expect(resolve).toBe(nodeResolve)
  expect(runtime.readFile()).toBe(42)
})
