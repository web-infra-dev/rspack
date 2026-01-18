import { require } from './require';

function run() {
  return require.resolve('module')
}

it('should evaluate `require.resolve`', () => {
  expect(run.toString()).toMatchSnapshot()
})
