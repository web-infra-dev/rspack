import webpackRuntimeAsset from './__webpack_require__.mjs'
import rspackRuntimeAsset from './rspackRequire.mjs'

new URL('./__webpack_require__.mjs', import.meta.url)
new URL('./rspackRequire.mjs', import.meta.url)

it('should deconflict asset imports from runtime bindings', () => {
  expect(webpackRuntimeAsset).toBe('webpack runtime asset')
  expect(rspackRuntimeAsset).toBe('rspack runtime asset')
})

export { rspackRuntimeAsset, webpackRuntimeAsset }
