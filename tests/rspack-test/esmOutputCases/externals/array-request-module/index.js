import fsPromises from 'fs-promises'

it('should support array-type module external with property access', async () => {
  const fs = await import(/* webpackIgnore: true */ 'node:fs')
  expect(fsPromises).toBe(fs.promises)
})
