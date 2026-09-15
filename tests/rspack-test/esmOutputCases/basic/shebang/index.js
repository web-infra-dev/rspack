#!/usr/bin/env node

it('should have shebang in entry chunk', async () => {
  const fs = await import(/* webpackIgnore: true */ 'node:fs')
  const path = await import(/* webpackIgnore: true */ 'node:path')

  const code = fs.readFileSync(path.join(__dirname, 'main.mjs'), 'utf-8')

  expect(code.startsWith('#!')).toBeTruthy()
})
