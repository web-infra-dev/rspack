import './index.css'
import { name } from './components/button'
import './components/button/index.css'

it('should preserve modules with native css', () => {
  expect(name).toBe('button')
})
