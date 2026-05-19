import './index.css'
import { name } from './components/button'
import './components/button/index.css'

it('should preserve modules with css-extract', () => {
  expect(name).toBe('button')
})
