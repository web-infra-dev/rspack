import b from './b.asset.mjs'
import a from './a.asset.mjs'

new URL('./b.asset.mjs', import.meta.url)
new URL('./a.asset.mjs', import.meta.url)

export { a, b }
