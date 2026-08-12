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
      source.matchAll(/^import (__rspack_asset_[0-9a-f]+) from /gm),
      match => match[1],
    )

    expect(importedSymbols).toEqual([
      '__rspack_asset_28d349827cf20a88',
      '__rspack_asset_a1470a0d5a9b15a8',
    ])
    expect(new Set(importedSymbols).size).toBe(importedSymbols.length)
    expect(source).toContain(
      'module.exports = __rspack_asset_28d349827cf20a88;',
    )
    expect(source).toContain(
      'module.exports = __rspack_asset_a1470a0d5a9b15a8;',
    )
    expect(source).toContain(
      "const index_rspack_asset_28d349827cf20a88 = 'first application value'",
    )
    expect(source).toContain(
      "const index_rspack_asset_a1470a0d5a9b15a8 = 'second application value'",
    )
    expect(source).toContain(
      '__rspack_asset_28d349827cf20a88: index_rspack_asset_28d349827cf20a88,',
    )
    expect(source).toContain(
      '__rspack_asset_a1470a0d5a9b15a8: index_rspack_asset_a1470a0d5a9b15a8,',
    )
    expect(source).not.toMatch(/\brequire\(/)
  },
}
