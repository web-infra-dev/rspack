const fs = require('node:fs')
const path = require('node:path')

module.exports = {
  snapshotFileFilter(file) {
    return file === 'main.mjs'
  },
  snapshotContent(content) {
    return content
      .split('\n')
      .filter(line =>
        /^(?:import|module\.exports|const|export ).*rspack_asset_/.test(line),
      )
      .join('\n')
  },
  afterExecute(options) {
    const source = fs.readFileSync(
      path.join(options.output.path, 'main.mjs'),
      'utf8',
    )
    const importedSymbols = Array.from(
      source.matchAll(/^import (__rspack_asset_\d+) from /gm),
      match => match[1],
    )

    expect(importedSymbols).toEqual(['__rspack_asset_1', '__rspack_asset_2'])
    expect(new Set(importedSymbols).size).toBe(importedSymbols.length)
    expect(source).toContain('module.exports = __rspack_asset_1;')
    expect(source).toContain('module.exports = __rspack_asset_2;')
    expect(source).toContain(
      "const index_rspack_asset_1 = 'first application value'",
    )
    expect(source).toContain(
      "const index_rspack_asset_2 = 'second application value'",
    )
    expect(source).toContain(
      'const applicationValues = { __rspack_asset_1: index_rspack_asset_1, __rspack_asset_2: index_rspack_asset_2 }',
    )
    expect(source).not.toMatch(/\brequire\(/)
  },
}
