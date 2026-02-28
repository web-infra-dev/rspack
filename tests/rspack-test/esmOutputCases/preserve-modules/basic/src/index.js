import demo from './demo'
import '../other1'

it('should preserve modules', () => {
  expect(demo()).toBe('demo')
})
