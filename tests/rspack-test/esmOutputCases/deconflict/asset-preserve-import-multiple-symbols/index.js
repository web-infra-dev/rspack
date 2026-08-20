import dashedAsset from './same-name.asset.mjs'
import underscoredAsset from './same_name.asset.mjs'

new URL('./same-name.asset.mjs', import.meta.url)
new URL('./same_name.asset.mjs', import.meta.url)

const same_name_asset = 'first application value'
const same_name_asset_0 = 'second application value'
const applicationValues = {
  same_name_asset,
  same_name_asset_0,
}

it('should keep multiple asset import symbols unique and rename conflicts', () => {
  expect(dashedAsset).toBe('dashed asset')
  expect(underscoredAsset).toBe('underscored asset')
  expect(applicationValues).toEqual({
    same_name_asset: 'first application value',
    same_name_asset_0: 'second application value',
  })
})

export {
  same_name_asset as firstApplicationValue,
  same_name_asset_0 as secondApplicationValue,
  applicationValues,
  dashedAsset,
  underscoredAsset,
}
