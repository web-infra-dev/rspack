import { value } from './other'

__webpack_require__.p = ('/assets/')
__webpack_nonce__ ??= 'nonce'
__webpack_nonce__ ||= 'fallback'
__webpack_nonce__ &&= 'nonce-updated'
__webpack_public_path__ += ('nested/')

it('should have correct value', () => {
  const { rspackPublicPath } = import.meta
  expect(value()).toBe(42)
  expect(__webpack_public_path__).toBe('/assets/nested/')
  expect(rspackPublicPath).toBe('/assets/nested/')
  expect(__webpack_require__.custom).toBeUndefined()
  expect(__webpack_nonce__).toBe('nonce-updated')
})
