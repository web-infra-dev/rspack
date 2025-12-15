import { value as value1 } from './module.js'
import { value as value2 } from './module.mjs'

it('should have correct interop', () => {
  expect(value1()).toBe(24)
  expect(value2()).toBe(42)
})