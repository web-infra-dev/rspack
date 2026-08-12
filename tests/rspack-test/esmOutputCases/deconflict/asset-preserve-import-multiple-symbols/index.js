import dashedAsset from './same-name.asset.mjs'
import underscoredAsset from './same_name.asset.mjs'

new URL('./same-name.asset.mjs', import.meta.url)
new URL('./same_name.asset.mjs', import.meta.url)

const __rspack_asset_28d349827cf20a88 = 'first application value'
const __rspack_asset_a1470a0d5a9b15a8 = 'second application value'
const applicationValues = {
  __rspack_asset_28d349827cf20a88,
  __rspack_asset_a1470a0d5a9b15a8,
}

it('should keep multiple asset import symbols unique and rename conflicts', () => {
  expect(dashedAsset).toBe('dashed asset')
  expect(underscoredAsset).toBe('underscored asset')
  expect(applicationValues).toEqual({
    __rspack_asset_28d349827cf20a88: 'first application value',
    __rspack_asset_a1470a0d5a9b15a8: 'second application value',
  })
})

export {
  __rspack_asset_28d349827cf20a88 as firstApplicationValue,
  __rspack_asset_a1470a0d5a9b15a8 as secondApplicationValue,
  applicationValues,
  dashedAsset,
  underscoredAsset,
}
