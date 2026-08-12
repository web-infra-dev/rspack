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

    expect(source).toContain(
      'import __rspack_asset_6763b85de2fc2546 from "./assets/value.asset.mjs";',
    )
    expect(source).toContain(
      'module.exports = __rspack_asset_6763b85de2fc2546;',
    )
    expect(source).toContain(
      "const index_rspack_asset_6763b85de2fc2546 = 'application value'",
    )
    expect(source).toContain(
      'const shorthand = { __rspack_asset_6763b85de2fc2546: index_rspack_asset_6763b85de2fc2546 }',
    )
    expect(source).not.toMatch(/\brequire\(/)
  },
}
