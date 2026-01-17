import { require } from './require';

function run() {
  return require.resolve('@types/node')
}

it('should evaluate `require.resolve`', () => {
  expect(run()).toMatchSnapshot()
})
