import { require } from './require'

function run() {
  return require.resolve('module')
}

it('should not evaluate `require.resolve()` as `require()`', () => {
  expect(run.toString()).toContain(`.resolve('module')`)
})
