import assetValue from './value.asset.mjs'

const assetUrl = new URL('./value.asset.mjs', import.meta.url)

it('should preserve an asset import in a non-concatenated module', () => {
  expect(assetValue).toBe('asset value')
  expect(assetUrl.pathname.endsWith('/assets/value.asset.mjs')).toBe(true)
})

export { assetUrl, assetValue }
