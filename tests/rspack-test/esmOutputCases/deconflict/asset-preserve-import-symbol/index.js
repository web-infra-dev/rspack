import assetValue from './value.asset.mjs'

new URL('./value.asset.mjs', import.meta.url)

const __rspack_asset_1 = 'application value'
const shorthand = { __rspack_asset_1 }

function readApplicationValue() {
  return __rspack_asset_1
}

it('should rename an application symbol that conflicts with an asset import', () => {
  expect(assetValue).toBe('asset value')
  expect(readApplicationValue()).toBe('application value')
  expect(shorthand.__rspack_asset_1).toBe('application value')
})

export { __rspack_asset_1 as applicationValue, assetValue, shorthand }
