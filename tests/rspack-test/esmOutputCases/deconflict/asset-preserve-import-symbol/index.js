import assetValue from './value.asset.mjs'

new URL('./value.asset.mjs', import.meta.url)

const value_asset = 'application value'
const shorthand = { value_asset }

function readApplicationValue() {
  return value_asset
}

it('should rename an application symbol that conflicts with an asset import', () => {
  expect(assetValue).toBe('asset value')
  expect(readApplicationValue()).toBe('application value')
  expect(shorthand.value_asset).toBe('application value')
})

export { value_asset as applicationValue, assetValue, shorthand }
