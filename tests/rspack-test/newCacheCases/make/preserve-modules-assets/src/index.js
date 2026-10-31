import rootIcon from './index.png'
import { name, icon as buttonIcon } from './components/button'

it('should preserve modules with asset modules (same-name js + png)', () => {
  expect(name).toBe('button')
  // preserveModules emits assets at their source paths.
  expect(rootIcon).toContain('index.png')
  expect(buttonIcon).toContain('components/button/index.png')
})
