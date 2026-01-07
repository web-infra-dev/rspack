// import default from cjs
// js/esm: the default is module.exports of cjs
// js/auto: check __esModule in runtime
//        if so, module.exports.default, otherwise module.exports
//        otherwise, module.exports
export { default as value } from './foo.cjs'

it('should have correct interop for cjs', async () => {
  const { value } = await import(/*webpackIgnore: true*/ './main.mjs')
  expect(value).toBe(42)
})