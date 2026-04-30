import { readFile } from 'virtual-fs'

export { readFile }

it('should scope-hoist remapped named imports from external modules', async () => {
  const fs = await import(/* webpackIgnore: true */ 'fs')

  expect(readFile).toBe(fs.readFile)
})
