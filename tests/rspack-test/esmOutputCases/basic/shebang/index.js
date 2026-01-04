#!/usr/bin/env node


it('should have shebang in entry chunk', () => {
  const fs = __non_webpack_require__('fs')
  const path = __non_webpack_require__('path')

  const code = fs.readFileSync(path.join(__dirname, 'main.mjs'), 'utf-8')

  expect(code.startsWith('#!')).toBeTruthy()
})