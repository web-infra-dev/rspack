import asset from './mixed-asset.js'

new URL('./mixed-asset.js', import.meta.url)
globalThis.mixedAsset = asset
