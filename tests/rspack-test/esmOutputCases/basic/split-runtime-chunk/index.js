import { value } from './other'
import './runtime-writes'

it('should have correct value', () => {
  const { rspackPublicPath } = import.meta
  expect(value()).toBe(42)
  expect(__webpack_public_path__).toBe('/assets/nested/')
  expect(rspackPublicPath).toBe('/assets/nested/')
  expect(__webpack_require__.custom).toBeUndefined()
  expect(__webpack_nonce__).toBe('nonce-updated')
})
