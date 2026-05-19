import { value } from './lib'
import '../outside'

it('should not produce duplicate chunk names when entry module is moved', () => {
  expect(value).toBe('lib')
})
