import { value } from './other'

__webpack_require__.p = '/assets/'

it('should have correct value', () => {
  expect(value()).toBe(42)
  expect(__webpack_require__.p).toBe('/assets/')
})
