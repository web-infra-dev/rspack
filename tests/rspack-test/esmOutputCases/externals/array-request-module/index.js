import fsPromises from 'fs-promises'

it('should support array-type module external with property access', async () => {
  const fs = __non_webpack_require__('fs')
  expect(fsPromises).toBe(fs.promises)
})
